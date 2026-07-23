use std::sync::Arc;
use std::time::Duration;

use common::error::Error;
use common::network::Network;
use common::{community::Community, error::network, message::Message, user::User};
use quinn::{
    Endpoint,
    crypto::rustls::QuicClientConfig,
    rustls::{
        self,
        client::danger::{ServerCertVerified, ServerCertVerifier},
    },
};
use tokio::sync::{mpsc, oneshot};

use crate::{ClientConfig, request::ClientRequest};

const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(1);
const MAX_IDLE_TIMEOUT: Duration = Duration::from_secs(3);

const CHANNEL_SIZE: usize = 32;
const RECONNECT_DELAY: Duration = Duration::from_secs(1);

pub async fn connect(server_address: &str, server_name: &str) -> Result<ClientRequest, Error> {
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(config());

    let connection = endpoint
        .connect(server_address.parse().unwrap(), server_name)?
        .await?;

    Ok(ClientRequest::new(connection))
}

fn config() -> quinn::ClientConfig {
    let crypto_provider = Arc::new(rustls::crypto::ring::default_provider());
    let tls_config = rustls::ClientConfig::builder_with_provider(crypto_provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .unwrap()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyCertificate(crypto_provider)))
        .with_no_client_auth();

    let mut client_config =
        quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(tls_config).unwrap()));

    let mut transport_config = quinn::TransportConfig::default();
    transport_config.keep_alive_interval(Some(KEEP_ALIVE_INTERVAL));
    transport_config.max_idle_timeout(Some(MAX_IDLE_TIMEOUT.try_into().unwrap()));

    client_config.transport_config(Arc::new(transport_config));

    client_config
}

macro_rules! network_commands {
    ($($variant:ident => $method:ident($($arg:ident), *) ->  $ret:ty),* $(,)?) => {
        #[derive(Debug)]
        pub enum NetworkCommand {
            $($variant{
                $($arg: String, )*
                reply: oneshot::Sender<Result<$ret, Error>>,
            },)*
        }

        impl NetworkHandle {
            $(pub async fn $method(&self, $($arg: &str),*) -> Result<$ret, Error> {
                let (reply, response) = oneshot::channel();
                let command = NetworkCommand::$variant {
                    $($arg: $arg.to_string(),)*
                    reply,
                };

                self.command_sender.send(command).await.map_err(|_| network::Error::ChannelClosed)?;

                response.await.map_err(|_| network::Error::ChannelClosed)?
            })*
        }

        async fn dispatch(client_request: ClientRequest, command: NetworkCommand) {
            match command {
                $(NetworkCommand::$variant { $($arg,)* reply} => {
                   let _ = reply.send(client_request.$method($(&$arg),*).await);
                })*
            }
        }
    };
}

network_commands! {
    Authenticate => authenticate(authentication_token) -> User,
    Deauthenticate => deauthenticate(authentication_token) -> User,
    CreateUser => create_user(username) -> User,
    ReadUser => read_user(user_id) -> User,
    UpdateUser => update_user(user_id, username) -> User,
    DeleteUser => delete_user(user_id) -> User,
    CreateCommunity => create_community(community_name) -> Community,
    ReadCommunity => read_community(community_id) -> Community,
    UpdateCommunity => update_community(community_id, community_name) -> Community,
    DeleteCommunity => delete_community(community_id) -> Community,
    CreateMessage => create_message(user_id, community_id, message_body) -> Message,
    ReadMessage => read_message(message_id) -> Message,
    UpdateMessage => update_message(message_id, message_body) -> Message,
    DeleteMessage => delete_message(message_id) -> Message,
    JoinCommunity => join_community(user_id, community_id) -> (),
    LeaveCommunity => leave_community(user_id, community_id) -> (),
    IsUserIn => is_user_in(user_id, community_id) -> bool,
    UsersIn => users_in(community_id) -> Vec<String>,
    CommunityOf => community_of(user_id) -> Vec<String>,
}

#[derive(Debug)]
pub enum NetworkEvent {
    Connected(Option<User>),
    ConnectionFailed(Error),
    Disconnected(Error),
    MessageReceived(Message),
}

#[derive(Debug, Clone)]
pub struct NetworkHandle {
    command_sender: mpsc::Sender<NetworkCommand>,
}

impl NetworkHandle {
    fn new(command_sender: mpsc::Sender<NetworkCommand>) -> Self {
        Self { command_sender }
    }
}

pub fn start(client_config: ClientConfig) -> (NetworkHandle, mpsc::Receiver<NetworkEvent>) {
    let (command_sender, command_receiver) = mpsc::channel(CHANNEL_SIZE);
    let (event_sender, event_receiver) = mpsc::channel(CHANNEL_SIZE);

    std::thread::Builder::new()
        .name("network".to_string())
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("network runtime")
                .block_on(actor(client_config, command_receiver, event_sender));
        })
        .expect("network thread");

    (NetworkHandle::new(command_sender), event_receiver)
}

async fn actor(
    client_config: ClientConfig,
    mut command_receiver: mpsc::Receiver<NetworkCommand>,
    event_sender: mpsc::Sender<NetworkEvent>,
) {
    let mut authentication_token: Option<String> = None;
    'session: loop {
        let client_request = loop {
            match connect(&client_config.server_address, &client_config.server_name).await {
                Ok(client_request) => break client_request,
                Err(error_value) => {
                    if event_sender
                        .send(NetworkEvent::ConnectionFailed(error_value))
                        .await
                        .is_err()
                    {
                        return;
                    }

                    tokio::time::sleep(RECONNECT_DELAY).await;
                }
            };
        };

        let mut user = None;
        if let Some(authentication_token_) = &authentication_token {
            match client_request.authenticate(authentication_token_).await {
                Ok(user_) => user = Some(user_),
                Err(_) => authentication_token = None,
            }
        }

        if event_sender
            .send(NetworkEvent::Connected(user))
            .await
            .is_err()
        {
            return;
        }

        tokio::spawn(message_receive_loop(
            client_request.clone(),
            event_sender.clone(),
        ));

        loop {
            tokio::select! {
                command = command_receiver.recv() => match command {
                    Some(command) => {
                        match &command {
                            NetworkCommand::Authenticate { authentication_token: authentication_token_, .. } => {
                                authentication_token = Some(authentication_token_.to_owned());
                            }
                            NetworkCommand::Deauthenticate { .. } => {
                                authentication_token = None;
                            }
                            _ => {}
                        }
                        tokio::spawn(dispatch(client_request.clone(), command));
                    }
                    None => break 'session,
                },
                error_value = client_request.closed() => {
                    if event_sender.send(NetworkEvent::Disconnected(error_value.into())).await.is_err() {
                        return ;
                    }

                    break;
                }
            }
        }
    }
}

async fn message_receive_loop(
    client_request: ClientRequest,
    event_sender: mpsc::Sender<NetworkEvent>,
) {
    let Ok(mut receive_stream) = client_request.accept_unidirectional_stream().await else {
        return;
    };

    while let Ok(message) = Network::receive_message(&mut receive_stream).await {
        if event_sender
            .send(NetworkEvent::MessageReceived(message))
            .await
            .is_err()
        {
            return;
        }
    }
}

#[derive(Debug)]
struct AcceptAnyCertificate(Arc<rustls::crypto::CryptoProvider>);

impl ServerCertVerifier for AcceptAnyCertificate {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}
