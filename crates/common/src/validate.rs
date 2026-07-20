use crate::constant::{
    MAXIMUM_COMMUNITY_NAME_LENGTH, MAXIMUM_MESSAGE_LENGTH, MAXIMUM_USERNAME_LENGTH,
    MINIMUM_COMMUNITY_NAME_LENGTH, MINIMUM_MESSAGE_LENGTH, MINIMUM_USERNAME_LENGTH,
};

type UserError = crate::error::user::Error;
type MessageError = crate::error::message::Error;
type CommunityError = crate::error::community::Error;

pub fn validate_username(username: &str) -> Result<(), UserError> {
    if !username.is_ascii() {
        return Err(UserError::ASCII);
    }

    if username.len() < MINIMUM_USERNAME_LENGTH {
        return Err(UserError::MinimumLength(username.len()));
    }

    if username.len() > MAXIMUM_USERNAME_LENGTH {
        return Err(UserError::MaximumLength(username.len()));
    }

    Ok(())
}

pub fn validate_message_body(message: &str) -> Result<(), MessageError> {
    if message.len() < MINIMUM_MESSAGE_LENGTH {
        return Err(MessageError::MinimumLength(message.len()));
    }

    if message.len() > MAXIMUM_MESSAGE_LENGTH {
        return Err(MessageError::MaximumLength(message.len()));
    }

    Ok(())
}

pub fn validate_community_name(community_name: &str) -> Result<(), CommunityError> {
    if !community_name.is_ascii() {
        return Err(CommunityError::ASCII);
    }

    if community_name.len() < MINIMUM_COMMUNITY_NAME_LENGTH {
        return Err(CommunityError::MinimumLength(community_name.len()));
    }

    if community_name.len() > MAXIMUM_COMMUNITY_NAME_LENGTH {
        return Err(CommunityError::MaximumLength(community_name.len()));
    }

    Ok(())
}
