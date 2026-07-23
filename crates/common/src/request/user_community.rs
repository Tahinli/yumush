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

#[derive(Debug, Clone, Encode, Decode)]
pub struct IsUserIn {
    user_id: String,
    community_id: String,
}

impl IsUserIn {
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
pub struct UsersIn {
    community_id: String,
}

impl UsersIn {
    pub fn new(community_id: &str) -> Self {
        let community_id = community_id.to_string();

        Self { community_id }
    }

    pub fn get_community_id(&self) -> &str {
        &self.community_id
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CommunityOf {
    user_id: String,
}

impl CommunityOf {
    pub fn new(user_id: &str) -> Self {
        let user_id = user_id.to_string();

        Self { user_id }
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }
}
