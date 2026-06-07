use surrealdb::types::Value;

use crate::{
    community::{Community, CommunityID},
    database_::{COMMUNITY_TABLE, DB, USER_COMMUNITY_TABLE, USER_TABLE},
    error::{Error, database},
    user::{User, UserID},
    user_community::{JoinCommunity, LeaveCommunity},
};

pub(super) async fn join(
    join_community: &JoinCommunity,
    database_connection: &DB,
) -> Result<(), Error> {
    let query = format!("RELATE $user_id->{USER_COMMUNITY_TABLE}-> $community_id;");
    database_connection
        .query(query)
        .bind(("user_id", join_community.get_user_id().as_record_id()))
        .bind((
            "community_id",
            join_community.get_community_id().as_record_id(),
        ))
        .await?
        .check()?;

    Ok(())
}

pub(super) async fn leave(
    leave_community: &LeaveCommunity,
    database_connection: &DB,
) -> Result<(), Error> {
    let query =
        format!("DELETE $user_id->{USER_COMMUNITY_TABLE} WHERE out = $community_id RETURN BEFORE;");
    let mut result = database_connection
        .query(query)
        .bind(("user_id", leave_community.get_user_id().as_record_id()))
        .bind((
            "community_id",
            leave_community.get_community_id().as_record_id(),
        ))
        .await?
        .check()?;

    let deleted = result.take::<Vec<Value>>(0)?;
    if deleted.is_empty() {
        return Err(database::Error::Delete.into());
    } else {
        Ok(())
    }
}

pub(super) async fn is_user_in(
    user: &User,
    community: &Community,
    database_connection: &DB,
) -> Result<bool, Error> {
    let query = format!(
        "RETURN count(SELECT 1 FROM {USER_COMMUNITY_TABLE} WHERE in = $user_id AND out = $community_id) > 0;"
    );
    let mut result = database_connection
        .query(query)
        .bind(("user_id", user.get_id().as_record_id()))
        .bind(("community_id", community.get_id().as_record_id()))
        .await?
        .check()?;

    let result = result.take::<Option<bool>>(0)?;

    Ok(result.unwrap_or(false))
}

pub(super) async fn users_in(
    community: &Community,
    database_connection: &DB,
) -> Result<Vec<UserID>, Error> {
    let query = format!("RETURN $community_id<-{USER_COMMUNITY_TABLE}<-{USER_TABLE};");
    let mut result = database_connection
        .query(query)
        .bind(("community_id", community.get_id().as_record_id()))
        .await?
        .check()?;

    let mut result = result.take::<Vec<UserID>>(0)?;
    result.sort();

    Ok(result)
}

pub(super) async fn communitys_of(
    user: &User,
    database_connection: &DB,
) -> Result<Vec<CommunityID>, Error> {
    let query = format!("RETURN $user_id->{USER_COMMUNITY_TABLE}->{COMMUNITY_TABLE};");
    let mut result = database_connection
        .query(query)
        .bind(("user_id", user.get_id().as_record_id()))
        .await?
        .check()?;

    let mut result = result.take::<Vec<CommunityID>>(0)?;
    result.sort();

    Ok(result)
}
