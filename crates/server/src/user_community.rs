use crate::{
    community::{Community, CommunityID},
    database_::DB,
    error::Error,
    user::{User, UserID},
};

mod database;

#[derive(Debug)]
pub enum CommunityEvent {
    JoinCommunity(JoinCommunity),
    LeaveCommunity(LeaveCommunity),
}

#[derive(Debug)]
pub struct JoinCommunity {
    user_id: UserID,
    community_id: CommunityID,
}

impl JoinCommunity {
    fn new(user_id: &UserID, community_id: &CommunityID) -> Self {
        let user_id = user_id.to_owned();
        let community_id = community_id.to_owned();

        Self {
            user_id,
            community_id,
        }
    }

    pub fn get_user_id(&self) -> &UserID {
        &self.user_id
    }

    pub fn get_community_id(&self) -> &CommunityID {
        &self.community_id
    }

    pub async fn apply(
        user_id: &UserID,
        community_id: &CommunityID,
        database_connection: &DB,
    ) -> Result<(), Error> {
        let join_community = Self::new(user_id, community_id);

        database::join(&join_community, database_connection).await
    }
}

#[derive(Debug)]
pub struct LeaveCommunity {
    user_id: UserID,
    community_id: CommunityID,
}

impl LeaveCommunity {
    fn new(user_id: &UserID, community_id: &CommunityID) -> Self {
        let user_id = user_id.to_owned();
        let community_id = community_id.to_owned();

        Self {
            user_id,
            community_id,
        }
    }

    pub fn get_user_id(&self) -> &UserID {
        &self.user_id
    }

    pub fn get_community_id(&self) -> &CommunityID {
        &self.community_id
    }

    pub async fn apply(
        user_id: &UserID,
        community_id: &CommunityID,
        database_connection: &DB,
    ) -> Result<(), Error> {
        let leave_community = Self::new(user_id, community_id);

        database::leave(&leave_community, database_connection).await
    }
}

pub async fn is_user_in(
    user: &User,
    community: &Community,
    database_connection: &DB,
) -> Result<bool, Error> {
    database::is_user_in(user, community, database_connection).await
}

pub async fn users_in(
    community: &Community,
    database_connection: &DB,
) -> Result<Vec<UserID>, Error> {
    database::users_in(community, database_connection).await
}
pub async fn communitys_of(
    user: &User,
    database_connection: &DB,
) -> Result<Vec<CommunityID>, Error> {
    database::communitys_of(user, database_connection).await
}

#[cfg(test)]
mod tests {
    use crate::{
        community::Community,
        error::{Error, database},
        test,
        user::User,
        user_community::{self, JoinCommunity, LeaveCommunity, is_user_in},
    };

    #[tokio::test]
    async fn join() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let user = User::create("Tahinli", &database_connection).await.unwrap();
        let community = Community::create("The Community", &database_connection)
            .await
            .unwrap();

        let join =
            JoinCommunity::apply(user.get_id(), community.get_id(), &database_connection).await;

        assert_eq!(true, join.is_ok());

        let is_user_in = is_user_in(&user, &community, &database_connection)
            .await
            .unwrap();

        assert!(is_user_in);
    }

    #[tokio::test]
    async fn join_again() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let user = User::create("Tahinli", &database_connection).await.unwrap();
        let community = Community::create("The Community", &database_connection)
            .await
            .unwrap();

        JoinCommunity::apply(user.get_id(), community.get_id(), &database_connection)
            .await
            .unwrap();

        let join =
            JoinCommunity::apply(user.get_id(), community.get_id(), &database_connection).await;

        assert!(join.is_err());
    }

    #[tokio::test]
    async fn leave() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let user = User::create("Tahinli", &database_connection).await.unwrap();
        let community = Community::create("The Community", &database_connection)
            .await
            .unwrap();
        JoinCommunity::apply(user.get_id(), community.get_id(), &database_connection)
            .await
            .unwrap();

        let leave =
            LeaveCommunity::apply(user.get_id(), community.get_id(), &database_connection).await;

        assert!(leave.is_ok());

        let is_user_in = is_user_in(&user, &community, &database_connection)
            .await
            .unwrap();

        assert!(!is_user_in);
    }

    #[tokio::test]
    async fn leave_again() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let user = User::create("Tahinli", &database_connection).await.unwrap();
        let community = Community::create("The Community", &database_connection)
            .await
            .unwrap();
        JoinCommunity::apply(user.get_id(), community.get_id(), &database_connection)
            .await
            .unwrap();

        LeaveCommunity::apply(user.get_id(), community.get_id(), &database_connection)
            .await
            .unwrap();

        let leave =
            LeaveCommunity::apply(user.get_id(), community.get_id(), &database_connection).await;

        assert!(matches!(
            leave,
            Err(Error::Database(database::Error::Delete))
        ));
    }

    #[tokio::test]
    async fn users_in() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let user_1 = User::create("Tahinli", &database_connection).await.unwrap();
        let user_2 = User::create("Tahinli", &database_connection).await.unwrap();
        let _user_3 = User::create("Tahinli", &database_connection).await.unwrap();
        let community = Community::create("The Community", &database_connection)
            .await
            .unwrap();

        JoinCommunity::apply(user_1.get_id(), community.get_id(), &database_connection)
            .await
            .unwrap();

        JoinCommunity::apply(user_2.get_id(), community.get_id(), &database_connection)
            .await
            .unwrap();

        let users_in = user_community::users_in(&community, &database_connection)
            .await
            .unwrap();
        let expected_result = vec![user_1.get_id().to_owned(), user_2.get_id().to_owned()];

        assert_eq!(users_in, expected_result)
    }

    #[tokio::test]
    async fn communitys_of() {
        let (database_connection, _temp_directory) = test::get_database().await;

        let user = User::create("Tahinli", &database_connection).await.unwrap();
        let community_1 = Community::create("The Community", &database_connection)
            .await
            .unwrap();
        let community_2 = Community::create("The Community", &database_connection)
            .await
            .unwrap();
        let _community_3 = Community::create("The Community", &database_connection)
            .await
            .unwrap();

        JoinCommunity::apply(user.get_id(), community_1.get_id(), &database_connection)
            .await
            .unwrap();

        JoinCommunity::apply(user.get_id(), community_2.get_id(), &database_connection)
            .await
            .unwrap();

        let communitys_of = user_community::communitys_of(&user, &database_connection)
            .await
            .unwrap();
        let expected_result = vec![
            community_1.get_id().to_owned(),
            community_2.get_id().to_owned(),
        ];

        assert_eq!(communitys_of, expected_result)
    }
}
