use crate::{database_::DB, error::Error, user::User};

pub(crate) async fn authenticate(
    authentication_token: &str,
    database_connection: &DB,
) -> Result<User, Error> {
    println!("{}", authentication_token);

    //todo
    //Err(Error::Common(common::error::Error::Authenticate))
    let user = User::read(&"0".into(), database_connection).await?;
    Ok(user)
}
