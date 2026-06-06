use surrealdb::{
    Surreal,
    engine::any::{self, Any},
    opt::auth::Root,
};

use crate::{ServerConfig, error::Error};

pub type DB = Surreal<Any>;

pub const USER_TABLE: &str = "user";
pub const COMMUNITY_TABLE: &str = "community";
pub const MESSAGE_TABLE: &str = "message";
pub const USER_COMMUNITY_TABLE: &str = "user_community";

pub async fn connect(server_config: &ServerConfig) -> Result<DB, Error> {
    let address = format!(
        "{}://{}",
        server_config.database_prefix, server_config.database_address
    );
    let database_connection = any::connect(address).await?;

    database_connection
        .use_ns(server_config.namespace.clone())
        .use_db(server_config.database.clone())
        .await?;

    if !server_config.username.is_empty() && !server_config.password.is_empty() {
        database_connection
            .signin(Root {
                username: server_config.username.clone(),
                password: server_config.password.clone(),
            })
            .await?;
    }

    Ok(database_connection)
}

pub async fn migrate(database_connection: &DB) -> Result<(), Error> {
    database_connection
        .query(format!(
            "
                DEFINE TABLE IF NOT EXISTS {USER_TABLE} SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS username ON {USER_TABLE} TYPE string;

                DEFINE TABLE IF NOT EXISTS {COMMUNITY_TABLE} SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS community_name ON {COMMUNITY_TABLE} TYPE string;

                DEFINE TABLE IF NOT EXISTS {MESSAGE_TABLE} SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS community_id ON {MESSAGE_TABLE} TYPE record<{COMMUNITY_TABLE}>;
                DEFINE FIELD IF NOT EXISTS user_id ON {MESSAGE_TABLE} TYPE record<{USER_TABLE}>;
                DEFINE FIELD IF NOT EXISTS message_body ON {MESSAGE_TABLE} TYPE string;
                DEFINE INDEX IF NOT EXISTS {MESSAGE_TABLE}_community_id ON {MESSAGE_TABLE} FIELDS community_id;
                DEFINE INDEX IF NOT EXISTS {MESSAGE_TABLE}_user_id ON {MESSAGE_TABLE} FIELDS user_id;

                DEFINE TABLE IF NOT EXISTS {USER_COMMUNITY_TABLE} TYPE RELATION FROM {USER_TABLE} TO {COMMUNITY_TABLE} ENFORCED SCHEMAFULL;
                DEFINE INDEX IF NOT EXISTS {USER_COMMUNITY_TABLE}_unique ON {USER_COMMUNITY_TABLE} FIELDS in, out UNIQUE
            "
        ))
        .await?
        .check()?;

    Ok(())
}

pub async fn health(database_connection: &DB) -> Result<(), Error> {
    Ok(database_connection.health().await?)
}

// pub fn user_record_id(key: impl Into<RecordIdKey>) -> RecordId {
//     RecordId::new(USER_TABLE, key)
// }

// pub fn community_record_id(key: impl Into<RecordIdKey>) -> RecordId {
//     RecordId::new(COMMUNITY_TABLE, key)
// }

// pub fn message_record_id(key: impl Into<RecordIdKey>) -> RecordId {
//     RecordId::new(MESSAGE_TABLE, key)
// }
