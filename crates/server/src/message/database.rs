use crate::{
    community::CommunityID,
    database_::{DB, MESSAGE_TABLE},
    error::{self, Error},
    message::{Message, MessageID},
    user::UserID,
};

pub(super) async fn create(message: Message, database_connection: &DB) -> Result<Message, Error> {
    let result = database_connection
        .create(message.get_id().as_record_id())
        .content(message)
        .await?;

    result.ok_or_else(|| error::database::Error::Create.into())
}

pub(super) async fn read(id: &MessageID, database_connection: &DB) -> Result<Message, Error> {
    let result = database_connection.select(id.as_record_id()).await?;

    result.ok_or_else(|| error::database::Error::Read.into())
}

pub(super) async fn read_by_community_id(
    community_id: &CommunityID,
    database_connection: &DB,
) -> Result<Vec<Message>, Error> {
    let mut result = database_connection
        .query(
            "
                SELECT * FROM type::table($table) WHERE community_id = $community_id;
            ",
        )
        .bind(("table", MESSAGE_TABLE))
        .bind(("community_id", community_id.as_record_id()))
        .await?
        .check()?;

    let result = result.take::<Vec<Message>>(0)?;

    Ok(result)
}

pub(super) async fn read_by_user_id(
    user_id: &UserID,
    database_connection: &DB,
) -> Result<Vec<Message>, Error> {
    let mut result = database_connection
        .query(
            "
                SELECT * FROM type::table($table) WHERE user_id = $user_id;
            ",
        )
        .bind(("table", MESSAGE_TABLE))
        .bind(("user_id", user_id.as_record_id()))
        .await?
        .check()?;

    let result = result.take::<Vec<Message>>(0)?;

    Ok(result)
}

pub(super) async fn update(message: Message, database_connection: &DB) -> Result<Message, Error> {
    let result = database_connection
        .update(message.get_id().as_record_id())
        .content(message)
        .await?;

    result.ok_or_else(|| error::database::Error::Update.into())
}

pub(super) async fn delete(id: MessageID, database_connection: &DB) -> Result<Message, Error> {
    let result = database_connection.delete(id.as_record_id()).await?;

    result.ok_or_else(|| error::database::Error::Delete.into())
}
