use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use common::{
    network::{AUTHENTICATION_MAX_READ_LENGHT, Network},
    request::Request,
    response::Response,
};
use quinn::{Connection, Endpoint, Incoming, RecvStream, SendStream};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_stream::{StreamExt, wrappers::ReceiverStream};

use crate::{
    authentication::authenticate,
    community::{Community, CommunityID},
    database_::DB,
    error::Error,
    request::handle_request,
    user::UserID,
    user_community::users_in,
};

const CHANNEL_LENGTH: usize = 2048;
const SESSION_LENGTH: usize = 64;

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(0);

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

    let (event_sender, event_receiver) = mpsc::channel(CHANNEL_LENGTH);
    let database_connection_clone = || -> DB { database_connection.clone() };

    tokio::spawn(the_actor(event_receiver, database_connection_clone()));

    while let Some(incoming) = endpoint.accept().await {
        let database_connection = database_connection.clone();
        let event_sender = event_sender.clone();

        tokio::spawn(handle_connection(
            incoming,
            event_sender,
            database_connection,
        ));
    }

    Ok(())
}

async fn handle_connection(
    incoming: Incoming,
    event_sender: mpsc::Sender<Event>,
    database_connection: DB,
) {
    match incoming.await {
        Ok(connection) => {
            if let Err(error_value) =
                establish_connection(&connection, event_sender, &database_connection).await
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
    event_sender: mpsc::Sender<Event>,
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

        let user_id = user.get_id().to_owned();
        tokio::spawn(listen(
            user_id,
            connection.clone(),
            event_sender.clone(),
            database_connection.clone(),
        ));

        Network::send_response(&response, send_stream).await?;

        let send_stream = connection.open_uni().await?;
        let _ = event_sender
            .send(Event::Joined(
                user.get_id().to_owned(),
                send_stream,
                connection.clone(),
            ))
            .await;

        Ok(())
    } else {
        Err(Error::Common(common::error::Error::Authenticate))
    }
}

async fn listen(
    user_id: UserID,
    connection: Connection,
    event_sender: mpsc::Sender<Event>,
    database_connection: DB,
) {
    let read_and_answer = async |user_id: UserID,
                                 send_stream: SendStream,
                                 receive_stream: RecvStream,
                                 event_sender: mpsc::Sender<Event>,
                                 database_connection: DB|
           -> Result<(), Error> {
        let request = Network::receive_request(receive_stream, None).await?;
        let response = handle_request(user_id, request, &database_connection).await;

        if let Response::CreateMessage(message) = &response {
            let _ = event_sender.send(Event::Sent(message.to_owned())).await;
        }

        Network::send_response(&response, send_stream).await?;

        Ok(())
    };

    while let Ok((send_stream, receive_stream)) = connection.accept_bi().await {
        let database_connection = database_connection.clone();
        let user_id = user_id.clone();

        tokio::spawn(read_and_answer(
            user_id,
            send_stream,
            receive_stream,
            event_sender.clone(),
            database_connection,
        ));
    }
}

enum Event {
    Joined(UserID, SendStream, Connection),
    Sent(common::message::Message),
}

type SessionID = u64;

struct Session {
    user_id: UserID,
    connection: Connection,
    message_sender: mpsc::Sender<Arc<common::message::Message>>,
    writer: JoinHandle<()>,
}

impl Session {
    fn new(
        user_id: UserID,
        connection: Connection,
        message_sender: mpsc::Sender<Arc<common::message::Message>>,
        writer: JoinHandle<()>,
    ) -> Self {
        Self {
            user_id,
            connection,
            message_sender,
            writer,
        }
    }
}

async fn write_session(
    mut message_receiver: mpsc::Receiver<Arc<common::message::Message>>,
    mut send_stream: SendStream,
) {
    while let Some(message) = message_receiver.recv().await {
        if let Err(error_value) = Network::send_message(&message, &mut send_stream).await {
            eprintln!("Error = Message | Session Write | {}", error_value);
            break;
        }
    }
}

async fn the_actor(event_receiver: mpsc::Receiver<Event>, database_connection: DB) {
    let mut sessions: HashMap<SessionID, Session> = HashMap::new();

    let mut events = ReceiverStream::new(event_receiver);

    while let Some(event) = events.next().await {
        sessions.retain(|_, session| {
            !session.writer.is_finished() && session.connection.close_reason().is_none()
        });

        match event {
            Event::Joined(user_id, send_stream, connection) => {
                let (message_sender, message_receiver) = mpsc::channel(SESSION_LENGTH);
                let writer = tokio::spawn(write_session(message_receiver, send_stream));

                sessions.insert(
                    NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed),
                    Session::new(user_id, connection, message_sender, writer),
                );
            }
            Event::Sent(message) => {
                let Ok(community) = Community::read(
                    &CommunityID::from(message.get_community_id()),
                    &database_connection,
                )
                .await
                else {
                    eprintln!(
                        "Error = Message | Community Read | {}",
                        message.get_community_id()
                    );
                    continue;
                };

                let Ok(users) = users_in(&community, &database_connection).await else {
                    eprintln!(
                        "Error = Message | Community Member Read | {}",
                        message.get_community_id()
                    );
                    continue;
                };

                let message = Arc::new(message);

                sessions.retain(|session_id, session| {
                    if !users.contains(&session.user_id) {
                        return true;
                    }

                    match session.message_sender.try_send(Arc::clone(&message)) {
                        Ok(()) => true,
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            eprintln!(
                                "Error = Message | Session Buffer is Full | {} | {}",
                                session_id,
                                session.user_id.as_str()
                            );
                            session.writer.abort();
                            false
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            session.writer.abort();
                            false
                        }
                    }
                });
            }
        }
    }
}
