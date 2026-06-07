use std::sync::LazyLock;

use common::validate::validate_community_name;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};
use tokio::sync::Mutex;
use ulid::Generator;

use crate::{
    database_::{COMMUNITY_TABLE, DB},
    error::Error,
};

mod database;

static ULID: LazyLock<Mutex<Generator>> = LazyLock::new(|| Generator::new().into());

#[derive(Debug, SurrealValue, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct CommunityID(RecordId);

impl CommunityID {
    async fn new() -> Self {
        loop {
            match ULID.lock().await.generate() {
                Ok(ulid) => return Self(RecordId::new(COMMUNITY_TABLE, ulid.to_string())),
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

impl ToOwned for CommunityID {
    type Owned = CommunityID;

    fn to_owned(&self) -> Self::Owned {
        Self(self.0.clone())
    }
}

impl From<&str> for CommunityID {
    fn from(value: &str) -> Self {
        Self(RecordId::new(COMMUNITY_TABLE, value))
    }
}

#[derive(Debug, SurrealValue, PartialEq, PartialOrd, Eq, Ord)]
pub struct CommunityName(String);

impl CommunityName {
    pub fn try_new(community_name: &str) -> Result<Self, Error> {
        let community_name = community_name.to_string();
        validate_community_name(&community_name)?;

        Ok(Self(community_name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ToOwned for CommunityName {
    type Owned = CommunityName;

    fn to_owned(&self) -> Self::Owned {
        Self(self.0.clone())
    }
}

#[derive(Debug, SurrealValue, PartialEq, PartialOrd, Eq, Ord)]
pub struct Community {
    id: CommunityID,
    community_name: CommunityName,
}

impl Community {
    async fn new(community_name: &str) -> Result<Self, Error> {
        let community_name = CommunityName::try_new(community_name)?;

        let id = CommunityID::new().await;

        Ok(Self { id, community_name })
    }

    pub fn get_id(&self) -> &CommunityID {
        &self.id
    }

    pub fn get_community_name(&self) -> &CommunityName {
        &self.community_name
    }

    fn update_name(mut self, community_name: &str) -> Result<Community, Error> {
        self.community_name = CommunityName::try_new(community_name)?;

        Ok(self)
    }

    pub async fn create(name: &str, database_connection: &DB) -> Result<Community, Error> {
        let community = Self::new(name).await?;

        database::create(community, database_connection).await
    }

    pub async fn read(id: &CommunityID, database_connection: &DB) -> Result<Community, Error> {
        database::read(id, database_connection).await
    }

    pub async fn read_by_name(
        name: &str,
        database_connection: &DB,
    ) -> Result<Vec<Community>, Error> {
        database::read_by_name(name, database_connection).await
    }

    pub async fn read_all(database_connection: &DB) -> Result<Vec<Community>, Error> {
        database::read_all(database_connection).await
    }

    pub async fn update(self, name: &str, database_connection: &DB) -> Result<Community, Error> {
        let user = self.update_name(name)?;
        database::update(user, database_connection).await
    }

    pub async fn delete(self, database_connection: &DB) -> Result<Community, Error> {
        database::delete(self.id, database_connection).await
    }
}

impl Into<common::community::Community> for Community {
    fn into(self) -> common::community::Community {
        common::community::Community::new(
            self.get_id().as_str(),
            self.get_community_name().as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        community::Community,
        error::{Error, database},
        test,
    };

    #[tokio::test]
    async fn create() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let name = "Tahinli";
        let community = Community::create(name, &database_connection).await;

        assert_eq!(name, community.unwrap().get_community_name().as_str());
    }

    #[tokio::test]
    async fn read() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let name = "Tahinli";
        let created_community = Community::create(name, &database_connection).await.unwrap();

        let searched_community = Community::read(&created_community.get_id(), &database_connection)
            .await
            .unwrap();

        assert_eq!(created_community, searched_community);
    }

    #[tokio::test]
    async fn read_by_name() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let name = "Tahinli";
        let community_1 = Community::create(name, &database_connection).await.unwrap();
        let community_2 = Community::create(name, &database_connection).await.unwrap();
        let _community_3 = Community::create("Not Tahinli", &database_connection)
            .await
            .unwrap();

        let expected_result = vec![community_1, community_2];

        let searched_communitys = Community::read_by_name(name, &database_connection)
            .await
            .unwrap();

        assert_eq!(expected_result, searched_communitys);
    }

    #[tokio::test]
    async fn read_all() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let name = "Tahinli";
        let community_1 = Community::create(name, &database_connection).await.unwrap();
        let community_2 = Community::create(name, &database_connection).await.unwrap();
        let community_3 = Community::create("Not Tahinli", &database_connection)
            .await
            .unwrap();

        let expected_result = vec![community_1, community_2, community_3];

        let searched_communitys = Community::read_all(&database_connection).await.unwrap();

        assert_eq!(expected_result, searched_communitys);
    }

    #[tokio::test]
    async fn update() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let name = "Tahinli";
        let created_community = Community::create(name, &database_connection).await.unwrap();
        let created_id = created_community.get_id().to_owned();
        let created_community_name = created_community.get_community_name().to_owned();

        let updated_community = created_community
            .update("Not Tahinli", &database_connection)
            .await
            .unwrap();

        let searched_community = Community::read(updated_community.get_id(), &database_connection)
            .await
            .unwrap();

        assert_eq!(&created_id, searched_community.get_id());
        assert_ne!(
            &created_community_name,
            searched_community.get_community_name()
        );
        assert_eq!(updated_community, searched_community);
    }

    #[tokio::test]
    async fn update_on_unpersistent_community() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let name = "Tahinli";
        let unpersistent_community = Community::new(name).await.unwrap();

        let updated_community = unpersistent_community
            .update("Not Tahinli", &database_connection)
            .await;

        assert!(matches!(
            updated_community,
            Err(Error::Database(database::Error::Update)),
        ));
    }

    #[tokio::test]
    async fn delete() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let name = "Tahinli";
        let created_community = Community::create(name, &database_connection).await.unwrap();
        let created_id = created_community.get_id().to_owned();
        let created_community_name = created_community.get_community_name().to_owned();

        let deleted_community = created_community
            .delete(&database_connection)
            .await
            .unwrap();

        assert_eq!(&created_id, deleted_community.get_id());
        assert_eq!(
            &created_community_name,
            deleted_community.get_community_name()
        );
    }
}
