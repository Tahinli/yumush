use common::utils::TOML;

use crate::network::serve;

pub mod authentication;
pub mod community;
pub(crate) mod database_;
pub(crate) mod error;
pub mod message;
pub mod network;
pub mod request;
pub mod user;
pub mod user_community;

#[cfg(test)]
mod test;

const SERVER_CONFIG_FILE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../configs/server_config.toml"
);

pub struct ServerConfig {
    database_prefix: String,
    database_address: String,
    namespace: String,
    database: String,
    username: String,
    password: String,
    server_address: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let mut toml = TOML::naive_toml_parser_from_file(SERVER_CONFIG_FILE_PATH).unwrap();

        Self {
            database_prefix: toml.fields.remove("database_prefix").unwrap(),
            database_address: toml.fields.remove("database_address").unwrap(),
            namespace: toml.fields.remove("namespace").unwrap(),
            database: toml.fields.remove("database").unwrap(),
            username: toml.fields.remove("username").unwrap(),
            password: toml.fields.remove("password").unwrap(),
            server_address: toml.fields.remove("server_address").unwrap(),
        }
    }
}

pub async fn showtime() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();
    let server_config = ServerConfig::default();

    let database_connection = database_::connect(&server_config).await.unwrap();
    database_::migrate(&database_connection).await.unwrap();
    database_::health(&database_connection).await.unwrap();

    serve(&server_config.server_address, &database_connection)
        .await
        .unwrap();
}
