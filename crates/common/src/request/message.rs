use bitcode::{Decode, Encode};

use crate::{error::Error, validate::validate_message_body};

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateMessage {
    community_id: String,
    user_id: String,
    message_body: String,
}

impl CreateMessage {
    pub fn new(user_id: &str, community_id: &str, message_body: &str) -> Result<Self, Error> {
        let user_id = user_id.to_string();
        let community_id = community_id.to_string();
        let message_body = message_body.to_string();
        validate_message_body(&message_body)?;

        Ok(Self {
            community_id,
            user_id,
            message_body,
        })
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

#[derive(Debug, Clone, Encode, Decode)]
pub struct ReadMessage {
    message_id: String,
}

impl ReadMessage {
    pub fn new(message_id: &str) -> Self {
        let message_id = message_id.to_string();

        Self { message_id }
    }

    pub fn get_message_id(&self) -> &str {
        &self.message_id
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateMessage {
    message_id: String,
    message_body: String,
}

impl UpdateMessage {
    pub fn new(message_id: &str, message_body: &str) -> Result<Self, Error> {
        let message_id = message_id.to_string();
        let message_body = message_body.to_string();
        validate_message_body(&message_body)?;

        Ok(Self {
            message_id,
            message_body,
        })
    }

    pub fn get_message_id(&self) -> &str {
        &self.message_id
    }

    pub fn get_message_body(&self) -> &str {
        &self.message_body
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DeleteMessage {
    message_id: String,
}

impl DeleteMessage {
    pub fn new(message_id: &str) -> Self {
        let message_id = message_id.to_string();

        Self { message_id }
    }

    pub fn get_message_id(&self) -> &str {
        &self.message_id
    }
}
