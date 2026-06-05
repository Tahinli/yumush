use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct Community {
    community_id: String,
    community_name: String,
}

impl Community {
    #[cfg(feature = "server")]
    pub fn new(community_id: &str, community_name: &str) -> Self {
        let community_id = community_id.to_string();
        let community_name = community_name.to_string();

        Self {
            community_id,
            community_name,
        }
    }

    pub fn get_community_id(&self) -> &str {
        &self.community_id
    }

    pub fn get_community_name(&self) -> &str {
        &self.community_name
    }
}
