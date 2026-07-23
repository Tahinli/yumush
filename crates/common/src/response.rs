use bitcode::{Decode, Encode};

use crate::{community::Community, error::Error, message::Message, user::User};

pub mod community;
pub mod message;
pub mod user;

#[derive(Debug, Clone, Encode, Decode)]
pub enum Response {
    Authentication(User),
    Deauthentication(User),
    CreateUser(User),
    ReadUser(User),
    UpdateUser(User),
    DeleteUser(User),
    CreateCommunity(Community),
    ReadCommunity(Community),
    UpdateCommunity(Community),
    DeleteCommunity(Community),
    CreateMessage(Message),
    ReadMessage(Message),
    UpdateMessage(Message),
    DeleteMessage(Message),
    JoinCommunity,
    LeaveCommunity,
    IsUserIn(bool),
    UsersIn(Vec<String>),
    CommunityOf(Vec<String>),
    Error(Error),
}
