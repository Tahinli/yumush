use common::{
    community::Community,
    error::Error,
    message::Message,
    network::Network,
    request::{
        Authentication, Deauthentication, Request,
        community::{CreateCommunity, DeleteCommunity, ReadCommunity, UpdateCommunity},
        message::{CreateMessage, DeleteMessage, ReadMessage, UpdateMessage},
        user::{CreateUser, DeleteUser, ReadUser, UpdateUser},
        user_community::{JoinCommunity, LeaveCommunity},
    },
    response::Response,
    user::User,
};

macro_rules! send_request_receive_response_then_classify {
    ($request:expr, $variant:path, $connection:expr) => {{
        let (send_stream, receive_stream) = $connection.open_bi().await?;
        let response =
            Network::send_request_and_receive_response(&$request, send_stream, receive_stream)
                .await?;
        match response {
            $variant(inner) => Ok(inner),
            Response::Error(error) => Err(error),
            _ => Err(Error::Server),
        }
    }};

    (@unit $request:expr, $variant:path, $connection:expr) => {{
        let (send_stream, receive_stream) = $connection.open_bi().await?;
        let response =
            Network::send_request_and_receive_response(&$request, send_stream, receive_stream)
                .await?;
        match response {
            $variant => Ok(()),
            Response::Error(error_value) => Err(error_value),
            _ => Err(Error::Server),
        }
    }};
}

#[derive(Debug, Clone)]
pub struct ClientRequest {
    connection: quinn::Connection,
}

impl ClientRequest {
    pub fn new(connection: quinn::Connection) -> Self {
        Self { connection }
    }

    pub async fn closed(&self) -> Error {
        self.connection.closed().await.into()
    }

    pub async fn authenticate(&self, authentication_token: &str) -> Result<User, Error> {
        let authentication = Authentication::new(authentication_token);
        let request = Request::Authentication(authentication);

        send_request_receive_response_then_classify!(
            request,
            Response::Authentication,
            self.connection
        )
    }

    pub async fn deauthenticate(&self, authentication_token: &str) -> Result<User, Error> {
        let deauthentication = Deauthentication::new(authentication_token);
        let request = Request::Deauthentication(deauthentication);

        send_request_receive_response_then_classify!(
            request,
            Response::Deauthentication,
            self.connection
        )
    }

    pub async fn create_user(&self, username: &str) -> Result<User, Error> {
        let create_user = CreateUser::new(username)?;
        let request = Request::CreateUser(create_user);

        send_request_receive_response_then_classify!(request, Response::CreateUser, self.connection)
    }

    pub async fn read_user(&self, user_id: &str) -> Result<User, Error> {
        let read_user = ReadUser::new(user_id);
        let request = Request::ReadUser(read_user);

        send_request_receive_response_then_classify!(request, Response::ReadUser, self.connection)
    }

    pub async fn update_user(&self, user_id: &str, username: &str) -> Result<User, Error> {
        let update_user = UpdateUser::new(user_id, username)?;
        let request = Request::UpdateUser(update_user);

        send_request_receive_response_then_classify!(request, Response::UpdateUser, self.connection)
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<User, Error> {
        let delete_user = DeleteUser::new(user_id);
        let request = Request::DeleteUser(delete_user);

        send_request_receive_response_then_classify!(request, Response::DeleteUser, self.connection)
    }

    pub async fn create_community(&self, community_name: &str) -> Result<Community, Error> {
        let create_community = CreateCommunity::new(community_name)?;
        let request = Request::CreateCommunity(create_community);

        send_request_receive_response_then_classify!(
            request,
            Response::CreateCommunity,
            self.connection
        )
    }

    pub async fn read_community(&self, community_id: &str) -> Result<Community, Error> {
        let read_community = ReadCommunity::new(community_id);
        let request = Request::ReadCommunity(read_community);

        send_request_receive_response_then_classify!(
            request,
            Response::ReadCommunity,
            self.connection
        )
    }

    pub async fn update_community(
        &self,
        community_id: &str,
        community_name: &str,
    ) -> Result<Community, Error> {
        let update_community = UpdateCommunity::new(community_id, community_name)?;
        let request = Request::UpdateCommunity(update_community);

        send_request_receive_response_then_classify!(
            request,
            Response::UpdateCommunity,
            self.connection
        )
    }

    pub async fn delete_community(&self, community_id: &str) -> Result<Community, Error> {
        let delete_community = DeleteCommunity::new(community_id);
        let request = Request::DeleteCommunity(delete_community);

        send_request_receive_response_then_classify!(
            request,
            Response::DeleteCommunity,
            self.connection
        )
    }

    pub async fn create_message(
        &self,
        user_id: &str,
        community_id: &str,
        message_body: &str,
    ) -> Result<Message, Error> {
        let create_message = CreateMessage::new(user_id, community_id, message_body)?;
        let request = Request::CreateMessage(create_message);

        send_request_receive_response_then_classify!(
            request,
            Response::CreateMessage,
            self.connection
        )
    }

    pub async fn read_message(&self, message_id: &str) -> Result<Message, Error> {
        let read_message = ReadMessage::new(message_id);
        let request = Request::ReadMessage(read_message);

        send_request_receive_response_then_classify!(
            request,
            Response::ReadMessage,
            self.connection
        )
    }

    pub async fn update_message(
        &self,
        message_id: &str,
        message_body: &str,
    ) -> Result<Message, Error> {
        let update_message = UpdateMessage::new(message_id, message_body)?;
        let request = Request::UpdateMessage(update_message);

        send_request_receive_response_then_classify!(
            request,
            Response::UpdateMessage,
            self.connection
        )
    }

    pub async fn delete_message(&self, message_id: &str) -> Result<Message, Error> {
        let delete_message = DeleteMessage::new(message_id);
        let request = Request::DeleteMessage(delete_message);

        send_request_receive_response_then_classify!(
            request,
            Response::DeleteMessage,
            self.connection
        )
    }

    pub async fn join_community(&self, user_id: &str, community_id: &str) -> Result<(), Error> {
        let join_community = JoinCommunity::new(user_id, community_id);
        let request = Request::JoinCommunity(join_community);

        send_request_receive_response_then_classify!(
            @unit request,
            Response::JoinCommunity,
            self.connection
        )
    }

    pub async fn leave_community(&self, user_id: &str, community_id: &str) -> Result<(), Error> {
        let leave_community = LeaveCommunity::new(user_id, community_id);
        let request = Request::LeaveCommunity(leave_community);

        send_request_receive_response_then_classify!(
            @unit request,
            Response::LeaveCommunity,
            self.connection
        )
    }
}
