use std::sync::LazyLock;

use common::validate::validate_username;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};
use tokio::sync::Mutex;
use ulid::Generator;

use crate::{
    database_::{DB, USER_TABLE},
    error::Error,
};

mod database;

static ULID: LazyLock<Mutex<Generator>> = LazyLock::new(|| Generator::new().into());

#[derive(Debug, SurrealValue, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct UserID(RecordId);

impl UserID {
    async fn new() -> Self {
        loop {
            match ULID.lock().await.generate() {
                Ok(ulid) => return Self(RecordId::new(USER_TABLE, ulid.to_string())),
                Err(_) => continue,
            }
        }
    }

    pub(crate) fn as_record_id(&self) -> RecordId {
        self.0.clone()
    }

    pub(crate) fn as_str(&self) -> &str {
        match &self.0.key {
            RecordIdKey::String(user_id) => user_id,
            _ => unreachable!("We shouldn't be here"),
        }
    }
}

impl ToOwned for UserID {
    type Owned = UserID;

    fn to_owned(&self) -> Self::Owned {
        Self(self.0.clone())
    }
}

impl From<&str> for UserID {
    fn from(value: &str) -> Self {
        Self(RecordId::new(USER_TABLE, value))
    }
}

#[derive(Debug, SurrealValue, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Username(String);

impl Username {
    pub fn try_new(username: &str) -> Result<Self, Error> {
        let username = username.to_string();
        validate_username(&username)?;

        Ok(Self(username))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ToOwned for Username {
    type Owned = Username;

    fn to_owned(&self) -> Self::Owned {
        Self(self.0.clone())
    }
}

#[derive(Debug, SurrealValue, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct User {
    id: UserID,
    username: Username,
}

impl User {
    pub fn get_id(&self) -> &UserID {
        &self.id
    }

    pub fn get_username(&self) -> &Username {
        &self.username
    }

    async fn new(username: &str) -> Result<User, Error> {
        let username = Username::try_new(username)?;
        let id = UserID::new().await;

        Ok(Self { id, username })
    }

    fn update_username(mut self, username: &str) -> Result<User, Error> {
        self.username = Username::try_new(username)?;

        Ok(self)
    }

    pub async fn create(username: &str, database_connection: &DB) -> Result<User, Error> {
        let user = Self::new(username).await?;

        database::create(user, database_connection).await
    }

    pub async fn read(id: &UserID, database_connection: &DB) -> Result<User, Error> {
        database::read(id, database_connection).await
    }

    pub async fn read_by_username(
        username: &str,
        database_connection: &DB,
    ) -> Result<Vec<User>, Error> {
        database::read_by_username(username, database_connection).await
    }

    pub async fn read_all(database_connection: &DB) -> Result<Vec<User>, Error> {
        database::read_all(database_connection).await
    }

    pub async fn update(self, username: &str, database_connection: &DB) -> Result<User, Error> {
        let user = self.update_username(username)?;
        database::update(user, database_connection).await
    }

    pub async fn delete(self, database_connection: &DB) -> Result<User, Error> {
        database::delete(self.id, database_connection).await
    }
}

impl Into<common::user::User> for User {
    fn into(self) -> common::user::User {
        common::user::User::new(self.get_id().as_str(), self.get_username().as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::{Error, database},
        test,
        user::User,
    };

    #[tokio::test]
    async fn create() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let user = User::create(username, &database_connection).await;

        assert_eq!(username, user.unwrap().get_username().as_str());
    }

    #[tokio::test]
    async fn read() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let created_user = User::create(username, &database_connection).await.unwrap();

        let searched_user = User::read(&created_user.get_id(), &database_connection)
            .await
            .unwrap();

        assert_eq!(created_user, searched_user);
    }

    #[tokio::test]
    async fn read_by_username() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let user_1 = User::create(username, &database_connection).await.unwrap();
        let user_2 = User::create(username, &database_connection).await.unwrap();
        let _user_3 = User::create("Not Tahinli", &database_connection)
            .await
            .unwrap();

        let expected_result = vec![user_1, user_2];

        let searched_users = User::read_by_username(username, &database_connection)
            .await
            .unwrap();

        assert_eq!(expected_result, searched_users);
    }

    #[tokio::test]
    async fn read_all() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let user_1 = User::create(username, &database_connection).await.unwrap();
        let user_2 = User::create(username, &database_connection).await.unwrap();
        let user_3 = User::create("Not Tahinli", &database_connection)
            .await
            .unwrap();

        let expected_result = vec![user_1, user_2, user_3];

        let searched_users = User::read_all(&database_connection).await.unwrap();

        assert_eq!(expected_result, searched_users);
    }

    #[tokio::test]
    async fn update() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let created_user = User::create(username, &database_connection).await.unwrap();
        let created_id = created_user.get_id().to_owned();
        let created_user_username = created_user.get_username().to_owned();

        let updated_user = created_user
            .update("Not Tahinli", &database_connection)
            .await
            .unwrap();

        let searched_user = User::read(updated_user.get_id(), &database_connection)
            .await
            .unwrap();

        assert_eq!(&created_id, searched_user.get_id());
        assert_ne!(&created_user_username, searched_user.get_username());
        assert_eq!(updated_user, searched_user);
    }

    #[tokio::test]
    async fn update_on_unpersistent_user() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let unperistent_user = User::new(username).await.unwrap();

        let updated_user = unperistent_user
            .update("Not Tahinli", &database_connection)
            .await;

        assert!(matches!(
            updated_user,
            Err(Error::Database(database::Error::Update)),
        ));
    }

    #[tokio::test]
    async fn delete() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let created_user = User::create(username, &database_connection).await.unwrap();
        let created_id = created_user.get_id().to_owned();
        let created_user_username = created_user.get_username().to_owned();

        let deleted_user = created_user.delete(&database_connection).await.unwrap();

        assert_eq!(&created_id, deleted_user.get_id());
        assert_eq!(&created_user_username, deleted_user.get_username());
    }
}
