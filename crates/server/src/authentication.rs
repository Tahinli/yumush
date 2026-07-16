use crate::{database_::DB, error::Error, user::User};

pub(crate) async fn authenticate(
    authentication_token: &str,
    database_connection: &DB,
) -> Result<User, Error> {
    println!("{}", authentication_token);

    //todo
    //Err(Error::Common(common::error::Error::Authenticate))
    let users = User::read_by_username(authentication_token, database_connection).await?;
    let user = users
        .into_iter()
        .next()
        .ok_or(Error::Common(common::error::Error::Authenticate))?;
    Ok(user)
}

pub(crate) async fn deauthenticate(
    authentication_token: &str,
    database_connection: &DB,
) -> Result<User, Error> {
    println!("{}", authentication_token);

    //todo
    //Err(Error::Common(common::error::Error::Authenticate))
    let users = User::read_by_username(authentication_token, database_connection).await?;
    let user = users
        .into_iter()
        .next()
        .ok_or(Error::Common(common::error::Error::Authenticate))?;
    Ok(user)
}
