use crate::{
    community::{Community, CommunityID},
    database_::{COMMUNITY_TABLE, DB},
    error::{self, Error},
};

pub(super) async fn create(
    community: Community,
    database_connection: &DB,
) -> Result<Community, Error> {
    let result = database_connection
        .create(community.get_id().as_record_id())
        .content(community)
        .await?;

    result.ok_or_else(|| error::database::Error::Create.into())
}

pub(super) async fn read(id: &CommunityID, database_connection: &DB) -> Result<Community, Error> {
    let result = database_connection.select(id.as_record_id()).await?;

    result.ok_or_else(|| error::database::Error::Read.into())
}

pub(super) async fn read_by_name(
    name: &str,
    database_connection: &DB,
) -> Result<Vec<Community>, Error> {
    let mut result = database_connection
        .query(
            "
                SELECT * FROM type::table($table) WHERE community_name = $community_name;
            ",
        )
        .bind(("table", COMMUNITY_TABLE))
        .bind(("community_name", name.to_string()))
        .await?
        .check()?;

    let result = result.take::<Vec<Community>>(0)?;

    Ok(result)
}

pub(super) async fn read_all(database_connection: &DB) -> Result<Vec<Community>, Error> {
    let result = database_connection.select(COMMUNITY_TABLE).await?;

    Ok(result)
}

pub(super) async fn update(
    community: Community,
    database_connection: &DB,
) -> Result<Community, Error> {
    let result = database_connection
        .update(community.get_id().as_record_id())
        .content(community)
        .await?;

    result.ok_or_else(|| error::database::Error::Update.into())
}

pub(super) async fn delete(id: CommunityID, database_connection: &DB) -> Result<Community, Error> {
    let result = database_connection.delete(id.as_record_id()).await?;

    result.ok_or_else(|| error::database::Error::Delete.into())
}
