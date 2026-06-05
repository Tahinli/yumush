use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct User {
    user_id: String,
    username: String,
}

impl User {
    #[cfg(feature = "server")]
    pub fn new(user_id: &str, username: &str) -> Self {
        let user_id = user_id.to_string();
        let username = username.to_string();

        Self { user_id, username }
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }
}
