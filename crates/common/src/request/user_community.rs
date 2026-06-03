use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct JoinCommunity {
    user_id: String,
    community_id: String,
}

impl JoinCommunity {
    pub fn new(user_id: &str, community_id: &str) -> Self {
        let user_id = user_id.to_string();
        let community_id = community_id.to_string();

        Self {
            user_id,
            community_id,
        }
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    pub fn get_community_id(&self) -> &str {
        &self.community_id
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct LeaveCommunity {
    user_id: String,
    community_id: String,
}

impl LeaveCommunity {
    pub fn new(user_id: &str, community_id: &str) -> Self {
        let user_id = user_id.to_string();
        let community_id = community_id.to_string();

        Self {
            user_id,
            community_id,
        }
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    pub fn get_community_id(&self) -> &str {
        &self.community_id
    }
}
