use common::{request::Request, response::Response};

use crate::{
    authentication::{authenticate, deauthenticate},
    community::Community,
    database_::DB,
    error::Error,
    message::Message,
    user::User,
    user_community::{JoinCommunity, LeaveCommunity, communitys_of, is_user_in, users_in},
};

pub async fn handle_request(request: Request, database_connection: &DB) -> Response {
    let request_matcher = async || -> Result<Response, Error> {
        match request {
            Request::Authentication(authentication) => {
                let user = authenticate(
                    authentication.get_authentication_token(),
                    database_connection,
                )
                .await?;

                Ok(Response::Authentication(user.into()))
            }
            Request::Deauthentication(deauthentication) => {
                let user = deauthenticate(
                    deauthentication.get_authentication_token(),
                    database_connection,
                )
                .await?;

                Ok(Response::Deauthentication(user.into()))
            }
            Request::CreateUser(create_user) => {
                let user = User::create(create_user.get_username(), database_connection).await?;

                Ok(Response::CreateUser(user.into()))
            }
            Request::ReadUser(read_user) => {
                let user = User::read(&read_user.get_user_id().into(), database_connection).await?;

                Ok(Response::ReadUser(user.into()))
            }
            Request::UpdateUser(update_user) => {
                let user =
                    User::read(&update_user.get_user_id().into(), database_connection).await?;
                let user = user
                    .update(update_user.get_username(), database_connection)
                    .await?;

                Ok(Response::UpdateUser(user.into()))
            }
            Request::DeleteUser(delete_user) => {
                let user =
                    User::read(&delete_user.get_user_id().into(), database_connection).await?;
                let user = user.delete(database_connection).await?;

                Ok(Response::DeleteUser(user.into()))
            }
            Request::CreateCommunity(create_community) => {
                let community =
                    Community::create(create_community.get_community_name(), database_connection)
                        .await?;

                Ok(Response::CreateCommunity(community.into()))
            }
            Request::ReadCommunity(read_community) => {
                let community = Community::read(
                    &read_community.get_community_id().into(),
                    database_connection,
                )
                .await?;

                Ok(Response::ReadCommunity(community.into()))
            }
            Request::UpdateCommunity(update_community) => {
                let community = Community::read(
                    &update_community.get_community_id().into(),
                    database_connection,
                )
                .await?;

                let community = community
                    .update(update_community.get_community_name(), database_connection)
                    .await?;

                Ok(Response::UpdateCommunity(community.into()))
            }
            Request::DeleteCommunity(delete_community) => {
                let community = Community::read(
                    &delete_community.get_community_id().into(),
                    database_connection,
                )
                .await?;

                let community = community.delete(database_connection).await?;

                Ok(Response::DeleteCommunity(community.into()))
            }
            Request::CreateMessage(create_message) => {
                let user =
                    User::read(&create_message.get_user_id().into(), database_connection).await?;
                let community = Community::read(
                    &create_message.get_community_id().into(),
                    database_connection,
                )
                .await?;
                let message = create_message.get_message_body();
                let message =
                    Message::create(&user, &community, &message, database_connection).await?;

                Ok(Response::CreateMessage(message.into()))
            }
            Request::ReadMessage(read_message) => {
                let message =
                    Message::read(&read_message.get_message_id().into(), database_connection)
                        .await?;

                Ok(Response::ReadMessage(message.into()))
            }
            Request::UpdateMessage(update_message) => {
                let message =
                    Message::read(&update_message.get_message_id().into(), database_connection)
                        .await?;

                let message = message
                    .update(update_message.get_message_body(), database_connection)
                    .await?;

                Ok(Response::UpdateMessage(message.into()))
            }
            Request::DeleteMessage(delete_message) => {
                let message =
                    Message::read(&delete_message.get_message_id().into(), database_connection)
                        .await?;

                let message = message.delete(database_connection).await?;

                Ok(Response::DeleteMessage(message.into()))
            }
            Request::JoinCommunity(join_community) => {
                JoinCommunity::apply(
                    &join_community.get_user_id().into(),
                    &join_community.get_community_id().into(),
                    database_connection,
                )
                .await?;

                Ok(Response::JoinCommunity)
            }
            Request::LeaveCommunity(leave_community) => {
                LeaveCommunity::apply(
                    &leave_community.get_user_id().into(),
                    &leave_community.get_community_id().into(),
                    database_connection,
                )
                .await?;

                Ok(Response::LeaveCommunity)
            }
            Request::IsUserIn(is_user_in_) => {
                let user =
                    User::read(&is_user_in_.get_user_id().into(), database_connection).await?;
                let community =
                    Community::read(&is_user_in_.get_community_id().into(), database_connection)
                        .await?;

                let is_user_in = is_user_in(&user, &community, database_connection).await?;

                Ok(Response::IsUserIn(is_user_in))
            }
            Request::UsersIn(users_in_) => {
                let community =
                    Community::read(&users_in_.get_community_id().into(), database_connection)
                        .await?;

                let users_in = users_in(&community, database_connection).await?;

                Ok(Response::UsersIn(
                    users_in
                        .iter()
                        .map(|user_id| user_id.as_str().to_string())
                        .collect(),
                ))
            }
            Request::CommunityOf(community_of_) => {
                let user =
                    User::read(&community_of_.get_user_id().into(), database_connection).await?;

                let community_of = communitys_of(&user, database_connection).await?;

                Ok(Response::CommunityOf(
                    community_of
                        .iter()
                        .map(|community_id| community_id.as_str().to_string())
                        .collect(),
                ))
            }
        }
    };

    match request_matcher().await {
        Ok(response) => response,
        Err(error_value) => Response::Error(error_value.into()),
    }
}
