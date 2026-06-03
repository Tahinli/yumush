use bitcode::{Decode, Encode};

use crate::{community::Community, message::Message, user::User};

#[derive(Debug, Clone, Encode, Decode)]
pub enum Event {
    MessageCreate(Message),
    MessageUpdate(Message),
    MessageDelete(Message),
    UserJoin(Community, User),
    UserLeave(Community, User),
    CommunityUpdate(Community),
    CommunityDelete(Community),
}
