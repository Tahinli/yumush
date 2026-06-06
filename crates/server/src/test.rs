#[cfg(test)]
use crate::{
    ServerConfig,
    database_::{self, DB},
};
use tempfile::TempDir;

pub async fn get_database() -> (DB, TempDir) {
    let temp_directory = TempDir::new().unwrap();

    let server_config = ServerConfig {
        database_prefix: "surrealkv".to_string(),
        database_address: temp_directory.path().display().to_string(),
        namespace: "yumush".to_string(),
        database: "yumush".to_string(),
        username: "".to_string(),
        password: "".to_string(),
        server_address: "127.0.0.1:2026".to_string(),
    };

    let database_connection = database_::connect(&server_config).await.unwrap();
    database_::migrate(&database_connection).await.unwrap();

    (database_connection, temp_directory)
}
