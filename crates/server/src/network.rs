use std::{collections::HashMap, sync::LazyLock};

use common::{
    network::{AUTHENTICATION_MAX_READ_LENGHT, Network},
    request::Request,
    response::Response,
};
use quinn::{Connection, Endpoint, Incoming, RecvStream, SendStream};
use tokio::sync::RwLock;

use crate::{
    authentication::authenticate, database_::DB, error::Error, request::handle_request, user::User,
};

static USER_SEND_STREAM_MAP: LazyLock<RwLock<HashMap<User, SendStream>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[cfg(feature = "rcgen")]
fn config() -> quinn::ServerConfig {
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

    let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate_der = CertificateDer::from(certificate.cert);
    let certificate_key = PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der());
    quinn::ServerConfig::with_single_cert(vec![certificate_der], certificate_key.into()).unwrap()
}

pub async fn serve(server_address: &str, database_connection: &DB) {
    let endpoint = Endpoint::server(config(), server_address.parse().unwrap()).unwrap();

    while let Some(incoming) = endpoint.accept().await {
        let database_connection = database_connection.clone();
        tokio::spawn(handle_connection(incoming, database_connection));
    }
}

async fn handle_connection(incoming: Incoming, database_connection: DB) {
    match incoming.await {
        Ok(connection) => {
            if let Err(error_value) = establish_connection(&connection, &database_connection).await
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
    database_connection: &DB,
) -> Result<(), Error> {
    let (send_stream, receive_stream) = connection.accept_bi().await?;

    let authentication_request =
        Network::receive_request(receive_stream, Some(AUTHENTICATION_MAX_READ_LENGHT)).await?;
    if let Request::Authentication(authentication) = authentication_request {
        let authentication_token = authentication.get_authentication_token();
        let user = authenticate(authentication_token, database_connection).await?;
        let response = Response::Authentication(common::user::User::new(
            user.get_id().as_str(),
            user.get_username().as_str(),
        ));
        tokio::spawn(listen(connection.clone(), database_connection.clone()));

        Network::send_response(&response, send_stream).await?;

        let send_stream = connection.open_uni().await?;
        USER_SEND_STREAM_MAP.write().await.insert(user, send_stream);
    }

    Err(Error::Common(common::error::Error::Authenticate))
}

async fn listen(connection: Connection, database_connection: DB) {
    let read_and_answer = async |send_stream: SendStream,
                                 receive_stream: RecvStream,
                                 database_connection: DB|
           -> Result<(), Error> {
        let request = Network::receive_request(receive_stream, None).await?;
        let response = handle_request(request, &database_connection).await;
        Network::send_response(&response, send_stream).await?;

        Ok(())
    };

    while let Ok((send_stream, receive_stream)) = connection.accept_bi().await {
        let database_connection = database_connection.clone();
        tokio::spawn(read_and_answer(
            send_stream,
            receive_stream,
            database_connection,
        ));
    }
}
