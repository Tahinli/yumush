use bitcode::{Decode, Encode};

use crate::{error::Error, validate::validate_username};

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateUser {
    username: String,
}

impl CreateUser {
    pub fn new(username: &str) -> Result<Self, Error> {
        let username = username.to_string();
        validate_username(&username)?;

        Ok(Self { username })
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ReadUser {
    user_id: String,
}

impl ReadUser {
    pub fn new(user_id: &str) -> Self {
        let user_id = user_id.to_string();

        Self { user_id }
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateUser {
    user_id: String,
    username: String,
}

impl UpdateUser {
    pub fn new(user_id: &str, username: &str) -> Result<Self, Error> {
        let user_id = user_id.to_string();
        let username = username.to_string();
        validate_username(&username)?;

        Ok(Self { user_id, username })
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DeleteUser {
    user_id: String,
}

impl DeleteUser {
    pub fn new(user_id: &str) -> Self {
        let user_id = user_id.to_string();

        Self { user_id }
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }
}
