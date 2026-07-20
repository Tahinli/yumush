use std::collections::HashMap;

use common::{
    network::{AUTHENTICATION_MAX_READ_LENGHT, Network},
    request::Request,
    response::Response,
};
use quinn::{Connection, Endpoint, Incoming, RecvStream, SendStream};
use tokio::sync::mpsc;

use crate::{
    authentication::authenticate,
    community::{Community, CommunityID},
    database_::DB,
    error::Error,
    request::handle_request,
    user::User,
    user_community::is_user_in,
};

const CHANNEL_LENGTH: usize = 2048;

#[cfg(feature = "rcgen")]
fn config() -> quinn::ServerConfig {
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

    let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate_der = CertificateDer::from(certificate.cert);
    let certificate_key = PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der());
    quinn::ServerConfig::with_single_cert(vec![certificate_der], certificate_key.into()).unwrap()
}

pub async fn serve(server_address: &str, database_connection: &DB) -> Result<(), Error> {
    let endpoint = Endpoint::server(config(), server_address.parse()?)?;

    let (user_and_send_stream_sender, user_and_send_stream_receiver) =
        mpsc::channel(CHANNEL_LENGTH);
    let (message_sender, message_receiver) = mpsc::channel(CHANNEL_LENGTH);

    let database_connection_clone = || -> DB { database_connection.clone() };

    tokio::spawn(the_actor(
        user_and_send_stream_receiver,
        message_receiver,
        database_connection_clone(),
    ));

    while let Some(incoming) = endpoint.accept().await {
        let database_connection = database_connection.clone();
        let user_and_send_stream_sender = user_and_send_stream_sender.clone();
        let message_sender = message_sender.clone();

        tokio::spawn(handle_connection(
            incoming,
            user_and_send_stream_sender,
            message_sender,
            database_connection,
        ));
    }

    Ok(())
}

async fn handle_connection(
    incoming: Incoming,
    user_and_send_stream_sender: mpsc::Sender<(User, SendStream)>,
    message_sender: mpsc::Sender<common::message::Message>,
    database_connection: DB,
) {
    match incoming.await {
        Ok(connection) => {
            if let Err(error_value) = establish_connection(
                &connection,
                user_and_send_stream_sender,
                message_sender,
                &database_connection,
            )
            .await
            {
                eprintln!("Error = Endpoint Accept | {}", error_value.to_string());
                connection.close(0u8.into(), b"kendine iyi bak");
            }
        }
        Err(error_value) => eprintln!("Error = Connection | {}", error_value.to_string()),
    }
}

async fn establish_connection(
    connection: &Connection,
    user_and_send_stream_sender: mpsc::Sender<(User, SendStream)>,
    message_sender: mpsc::Sender<common::message::Message>,
    database_connection: &DB,
) -> Result<(), Error> {
    let (send_stream, receive_stream) = connection.accept_bi().await?;

    let first_request =
        Network::receive_request(receive_stream, Some(AUTHENTICATION_MAX_READ_LENGHT)).await?;

    if let Request::Authentication(authentication) = first_request {
        let authentication_token = authentication.get_authentication_token();
        let user = authenticate(authentication_token, database_connection).await?;
        let response = Response::Authentication(common::user::User::new(
            user.get_id().as_str(),
            user.get_username().as_str(),
        ));
        tokio::spawn(listen(
            connection.clone(),
            message_sender,
            database_connection.clone(),
        ));

        Network::send_response(&response, send_stream).await?;

        let send_stream = connection.open_uni().await?;
        let _ = user_and_send_stream_sender.send((user, send_stream)).await;

        Ok(())
    } else {
        Err(Error::Common(common::error::Error::Authenticate))
    }
}

async fn listen(
    connection: Connection,
    message_sender: mpsc::Sender<common::message::Message>,
    database_connection: DB,
) {
    let read_and_answer = async |send_stream: SendStream,
                                 receive_stream: RecvStream,
                                 message_sender: mpsc::Sender<common::message::Message>,
                                 database_connection: DB|
           -> Result<(), Error> {
        let request = Network::receive_request(receive_stream, None).await?;
        let response = handle_request(request, &database_connection).await;

        if let Response::CreateMessage(message) = &response {
            let _ = message_sender.send(message.to_owned()).await;
        }

        Network::send_response(&response, send_stream).await?;

        Ok(())
    };

    while let Ok((send_stream, receive_stream)) = connection.accept_bi().await {
        let database_connection = database_connection.clone();
        tokio::spawn(read_and_answer(
            send_stream,
            receive_stream,
            message_sender.clone(),
            database_connection,
        ));
    }
}

async fn the_actor(
    mut user_and_send_stream_receiver: mpsc::Receiver<(User, SendStream)>,
    mut message_receiver: mpsc::Receiver<common::message::Message>,
    database_connection: DB,
) {
    let mut user_stream_map = HashMap::new();

    loop {
        tokio::select! {
            user_and_send_stream = user_and_send_stream_receiver.recv() => match user_and_send_stream {
                Some((user, send_stream)) => {
                    user_stream_map.insert(user, send_stream);
                },
                None => return,
            },
            message = message_receiver.recv() => match message {
                Some(message) => {
                    let Ok(community) = Community::read(&CommunityID::from(message.get_community_id()), &database_connection).await else {
                        continue;
                    };

                    let old_map = std::mem::take(&mut user_stream_map);

                    for (user, mut send_stream) in old_map {
                        let keep = if is_user_in(&user, &community, &database_connection).await.unwrap_or(false) {
                            Network::send_message(&message, &mut send_stream).await.is_ok()
                        } else {
                            true
                        };

                        if keep {
                            user_stream_map.insert(user, send_stream);
                        }
                    }

                },
                None => return,
            }
        }
    }
}
