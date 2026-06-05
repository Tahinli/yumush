use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct Message {
    message_id: String,
    user_id: String,
    community_id: String,
    message_body: String,
}

impl Message {
    #[cfg(feature = "server")]
    pub fn new(message_id: &str, user_id: &str, community_id: &str, message_body: &str) -> Self {
        let message_id = message_id.to_string();
        let user_id = user_id.to_string();
        let community_id = community_id.to_string();
        let message_body = message_body.to_string();

        Self {
            message_id,
            user_id,
            community_id,
            message_body,
        }
    }

    pub fn get_id(&self) -> &str {
        &self.message_id
    }

    pub fn get_community_id(&self) -> &str {
        &self.community_id
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    pub fn get_message_body(&self) -> &str {
        &self.message_body
    }
}
