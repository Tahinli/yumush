use bitcode::{Decode, Encode};

use crate::{error::Error, validate::validate_community_name};

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreateCommunity {
    community_name: String,
}

impl CreateCommunity {
    pub fn new(community_name: &str) -> Result<Self, Error> {
        let community_name = community_name.to_string();
        validate_community_name(&community_name)?;

        Ok(Self { community_name })
    }

    pub fn get_community_name(&self) -> &str {
        &self.community_name
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ReadCommunity {
    community_id: String,
}

impl ReadCommunity {
    pub fn new(community_id: &str) -> Self {
        let community_id = community_id.to_string();

        Self { community_id }
    }

    pub fn get_community_id(&self) -> &str {
        &self.community_id
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateCommunity {
    community_id: String,
    community_name: String,
}

impl UpdateCommunity {
    pub fn new(community_id: &str, community_name: &str) -> Result<Self, Error> {
        let community_id = community_id.to_string();
        let community_name = community_name.to_string();
        validate_community_name(&community_name)?;

        Ok(Self {
            community_id,
            community_name,
        })
    }

    pub fn get_community_id(&self) -> &str {
        &self.community_id
    }

    pub fn get_community_name(&self) -> &str {
        &self.community_name
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DeleteCommunity {
    community_id: String,
}

impl DeleteCommunity {
    pub fn new(community_id: &str) -> Self {
        let community_id = community_id.to_string();

        Self { community_id }
    }

    pub fn get_community_id(&self) -> &str {
        &self.community_id
    }
}
