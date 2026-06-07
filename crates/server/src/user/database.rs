use crate::{
    database_::{DB, USER_TABLE},
    error::{self, Error},
    user::{User, UserID},
};

pub(super) async fn create(user: User, database_connection: &DB) -> Result<User, Error> {
    let result = database_connection
        .create(user.get_id().as_record_id())
        .content(user)
        .await?;

    result.ok_or_else(|| error::database::Error::Create.into())
}

pub(super) async fn read(id: &UserID, database_connection: &DB) -> Result<User, Error> {
    let result = database_connection.select(id.as_record_id()).await?;

    result.ok_or_else(|| error::database::Error::Read.into())
}

pub(super) async fn read_by_username(
    username: &str,
    database_connection: &DB,
) -> Result<Vec<User>, Error> {
    let mut result = database_connection
        .query(
            "
                SELECT * FROM type::table($table) WHERE username = $username;
            ",
        )
        .bind(("table", USER_TABLE))
        .bind(("username", username.to_string()))
        .await?
        .check()?;

    let result = result.take::<Vec<User>>(0)?;

    Ok(result)
}

pub(super) async fn read_all(database_connection: &DB) -> Result<Vec<User>, Error> {
    let result = database_connection.select(USER_TABLE).await?;

    Ok(result)
}

pub(super) async fn update(user: User, database_connection: &DB) -> Result<User, Error> {
    let result = database_connection
        .update(user.get_id().as_record_id())
        .content(user)
        .await?;

    result.ok_or_else(|| error::database::Error::Update.into())
}

pub(super) async fn delete(id: UserID, database_connection: &DB) -> Result<User, Error> {
    let result = database_connection.delete(id.as_record_id()).await?;

    result.ok_or_else(|| error::database::Error::Delete.into())
}
