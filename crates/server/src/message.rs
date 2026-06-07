use std::sync::LazyLock;

use common::validate::validate_message_body;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};
use tokio::sync::Mutex;
use ulid::Generator;

use crate::{
    community::{Community, CommunityID},
    database_::{DB, MESSAGE_TABLE},
    error::Error,
    user::{User, UserID},
};

mod database;

static ULID: LazyLock<Mutex<Generator>> = LazyLock::new(|| Generator::new().into());

#[derive(Debug, SurrealValue, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MessageID(RecordId);

impl MessageID {
    async fn new() -> Self {
        loop {
            match ULID.lock().await.generate() {
                Ok(ulid) => return Self(RecordId::new(MESSAGE_TABLE, ulid.to_string())),
                Err(_) => continue,
            }
        }
    }

    pub(crate) fn as_record_id(&self) -> RecordId {
        self.0.clone()
    }

    pub fn as_str(&self) -> &str {
        match &self.0.key {
            RecordIdKey::String(user_id) => user_id,
            _ => unreachable!("We shouldn't be here"),
        }
    }
}

impl ToOwned for MessageID {
    type Owned = MessageID;

    fn to_owned(&self) -> Self::Owned {
        Self(self.0.clone())
    }
}

impl From<&str> for MessageID {
    fn from(value: &str) -> Self {
        Self(RecordId::new(MESSAGE_TABLE, value))
    }
}

#[derive(Debug, SurrealValue, PartialEq, PartialOrd, Eq, Ord)]
pub struct MessageBody(String);

impl MessageBody {
    pub fn try_new(message_body: &str) -> Result<Self, Error> {
        let message_body = message_body.to_string();
        validate_message_body(&message_body)?;

        Ok(Self(message_body))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl ToOwned for MessageBody {
    type Owned = MessageBody;

    fn to_owned(&self) -> Self::Owned {
        Self(self.0.clone())
    }
}

#[derive(Debug, SurrealValue, PartialEq, PartialOrd, Eq, Ord)]
pub struct Message {
    id: MessageID,
    user_id: UserID,
    community_id: CommunityID,
    message_body: MessageBody,
}

impl Message {
    async fn new(user: &User, community: &Community, message_body: &str) -> Result<Self, Error> {
        let message_body = MessageBody::try_new(message_body)?;

        let id = MessageID::new().await;
        let user_id = user.get_id().to_owned();
        let community_id = community.get_id().to_owned();
        //todo RecordID olan yerleri düzeltiyoruz
        Ok(Self {
            id,
            user_id,
            community_id,
            message_body,
        })
    }

    pub fn get_user_id(&self) -> &UserID {
        &self.user_id
    }

    pub fn get_community_id(&self) -> &CommunityID {
        &self.community_id
    }

    pub fn get_message_body(&self) -> &MessageBody {
        &self.message_body
    }

    pub fn get_id(&self) -> &MessageID {
        &self.id
    }

    fn set_message_body(mut self, message_body: &str) -> Result<Message, Error> {
        self.message_body = MessageBody::try_new(message_body)?;

        Ok(self)
    }

    pub async fn create(
        user: &User,
        community: &Community,
        message: &str,
        database_connection: &DB,
    ) -> Result<Self, Error> {
        let message = Self::new(user, community, message).await?;
        database::create(message, database_connection).await
    }

    pub async fn read(id: &MessageID, database_connection: &DB) -> Result<Message, Error> {
        database::read(id, database_connection).await
    }

    pub async fn read_by_community_id(
        community_id: &CommunityID,
        database_connection: &DB,
    ) -> Result<Vec<Message>, Error> {
        database::read_by_community_id(community_id, database_connection).await
    }

    pub async fn read_by_user_id(
        user_id: &UserID,
        database_connection: &DB,
    ) -> Result<Vec<Message>, Error> {
        database::read_by_user_id(user_id, database_connection).await
    }

    pub async fn update(
        self,
        message_body: &str,
        database_connection: &DB,
    ) -> Result<Message, Error> {
        let message_body = self.set_message_body(message_body)?;
        database::update(message_body, database_connection).await
    }

    pub async fn delete(self, database_connection: &DB) -> Result<Message, Error> {
        database::delete(self.id, database_connection).await
    }
}

impl Into<common::message::Message> for Message {
    fn into(self) -> common::message::Message {
        common::message::Message::new(
            self.get_id().as_str(),
            self.get_user_id().as_str(),
            self.get_community_id().as_str(),
            self.message_body.as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        community::Community,
        error::{Error, database},
        message::Message,
        test,
        user::User,
    };

    #[tokio::test]
    async fn create() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let name = username;

        let user = User::create(username, &database_connection).await.unwrap();
        let community = Community::create(name, &database_connection).await.unwrap();

        let message_ = name;
        let message = Message::create(&user, &community, message_, &database_connection)
            .await
            .unwrap();

        assert_eq!(community.get_id(), message.get_community_id());
        assert_eq!(user.get_id(), message.get_user_id());
        assert_eq!(message_, message.get_message_body().as_str());
    }

    #[tokio::test]
    async fn read() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let name = username;

        let user = User::create(username, &database_connection).await.unwrap();
        let community = Community::create(name, &database_connection).await.unwrap();

        let message_ = name;
        let created_message = Message::create(&user, &community, message_, &database_connection)
            .await
            .unwrap();

        let searched_message = Message::read(created_message.get_id(), &database_connection)
            .await
            .unwrap();

        assert_eq!(created_message, searched_message);
    }

    #[tokio::test]
    async fn read_by_community_id() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let name = username;

        let user_1 = User::create(username, &database_connection).await.unwrap();
        let user_2 = User::create("Not Tahinli", &database_connection)
            .await
            .unwrap();
        let community_1 = Community::create(name, &database_connection).await.unwrap();
        let community_2 = Community::create("Not Tahinli", &database_connection)
            .await
            .unwrap();

        let message_ = name;
        let created_message_1 =
            Message::create(&user_1, &community_1, message_, &database_connection)
                .await
                .unwrap();
        let created_message_2 =
            Message::create(&user_2, &community_1, message_, &database_connection)
                .await
                .unwrap();
        let _created_message_3 =
            Message::create(&user_1, &community_2, message_, &database_connection)
                .await
                .unwrap();

        let expected_result = vec![created_message_1, created_message_2];

        let searched_messages =
            Message::read_by_community_id(community_1.get_id(), &database_connection)
                .await
                .unwrap();

        assert_eq!(expected_result, searched_messages);
    }

    #[tokio::test]
    async fn read_by_user_id() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let name = username;

        let user_1 = User::create(username, &database_connection).await.unwrap();
        let user_2 = User::create("Not Tahinli", &database_connection)
            .await
            .unwrap();
        let community_1 = Community::create(name, &database_connection).await.unwrap();
        let community_2 = Community::create("Not Tahinli", &database_connection)
            .await
            .unwrap();

        let message_ = name;
        let created_message_1 =
            Message::create(&user_1, &community_1, message_, &database_connection)
                .await
                .unwrap();
        let created_message_2 =
            Message::create(&user_1, &community_2, message_, &database_connection)
                .await
                .unwrap();
        let _created_message_3 =
            Message::create(&user_2, &community_1, message_, &database_connection)
                .await
                .unwrap();

        let expected_result = vec![created_message_1, created_message_2];

        let searched_messages = Message::read_by_user_id(user_1.get_id(), &database_connection)
            .await
            .unwrap();

        assert_eq!(expected_result, searched_messages);
    }

    #[tokio::test]
    async fn update() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let name = username;

        let user = User::create(username, &database_connection).await.unwrap();
        let community = Community::create(name, &database_connection).await.unwrap();

        let message_ = name;
        let created_message = Message::create(&user, &community, message_, &database_connection)
            .await
            .unwrap();

        let created_message_id = created_message.get_id().to_owned();
        let created_message_user_id = created_message.get_user_id().to_owned();
        let created_message_community_id = created_message.get_community_id().to_owned();
        let created_message_message = created_message.get_message_body().to_owned();

        let updated_message = created_message
            .update("Not Tahinli", &database_connection)
            .await
            .unwrap();

        let searched_message = Message::read(updated_message.get_id(), &database_connection)
            .await
            .unwrap();

        assert_eq!(&created_message_id, updated_message.get_id());
        assert_eq!(
            &created_message_community_id,
            updated_message.get_community_id()
        );
        assert_eq!(&created_message_user_id, updated_message.get_user_id());
        assert_ne!(&created_message_message, updated_message.get_message_body());
        assert_eq!(updated_message, searched_message);
    }

    #[tokio::test]
    async fn update_on_unpersistent_message() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let name = username;

        let user = User::create(username, &database_connection).await.unwrap();
        let community = Community::create(name, &database_connection).await.unwrap();

        let message_ = name;
        let unpersistent_message = Message::new(&user, &community, message_).await.unwrap();
        let updated_message = unpersistent_message
            .update(message_, &database_connection)
            .await;

        assert!(matches!(
            updated_message,
            Err(Error::Database(database::Error::Update)),
        ));
    }

    #[tokio::test]
    async fn delete() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let username = "Tahinli";
        let name = username;

        let user = User::create(username, &database_connection).await.unwrap();
        let community = Community::create(name, &database_connection).await.unwrap();

        let message_ = name;
        let created_message = Message::create(&user, &community, message_, &database_connection)
            .await
            .unwrap();
        let created_message_id = created_message.get_id().to_owned();
        let created_message_user_id = created_message.get_user_id().to_owned();
        let created_message_community_id = created_message.get_community_id().to_owned();
        let created_message_message = created_message.get_message_body().to_owned();

        let deleted_message = created_message.delete(&database_connection).await.unwrap();

        assert_eq!(&created_message_id, deleted_message.get_id());
        assert_eq!(
            &created_message_community_id,
            deleted_message.get_community_id()
        );
        assert_eq!(&created_message_user_id, deleted_message.get_user_id());
        assert_eq!(&created_message_message, deleted_message.get_message_body());
    }
}
