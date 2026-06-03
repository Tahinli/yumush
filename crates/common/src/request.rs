use bitcode::{Decode, Encode};

use crate::request::{
    community::{CreateCommunity, DeleteCommunity, ReadCommunity, UpdateCommunity},
    message::{CreateMessage, DeleteMessage, ReadMessage, UpdateMessage},
    user::{CreateUser, DeleteUser, ReadUser, UpdateUser},
    user_community::{JoinCommunity, LeaveCommunity},
};

pub mod community;
pub mod message;
pub mod user;
pub mod user_community;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Authentication(String);

impl Authentication {
    pub fn get_authentication_token(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum Request {
    Authentication(Authentication),
    CreateUser(CreateUser),
    ReadUser(ReadUser),
    UpdateUser(UpdateUser),
    DeleteUser(DeleteUser),
    CreateCommunity(CreateCommunity),
    ReadCommunity(ReadCommunity),
    UpdateCommunity(UpdateCommunity),
    DeleteCommunity(DeleteCommunity),
    CreateMessage(CreateMessage),
    ReadMessage(ReadMessage),
    UpdateMessage(UpdateMessage),
    DeleteMessage(DeleteMessage),
    JoinCommunity(JoinCommunity),
    LeaveCommunity(LeaveCommunity),
}
