use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use deadpool::managed::Object;
use diesel::{associations::HasTable, pg::sql_types, prelude::*};
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, scoped_futures::ScopedFutureExt,
    AsyncConnection, AsyncPgConnection, RunQueryDsl,
};

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub creation_date: NaiveDateTime,
    pub last_login_date: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub creation_date: NaiveDateTime,
    pub last_login_date: NaiveDateTime,
}

type Connection = Object<AsyncDieselConnectionManager<AsyncPgConnection>>;

use crate::schema;

pub async fn get_or_create(
    connection: &mut Connection,
    user_name: &str,
    user_email: &str,
    date: &NaiveDateTime,
) -> Result<User> {
    Ok(connection
        .transaction(move |conn| {
            get_or_create_transaction(conn, user_name, user_email, date).scope_boxed()
        })
        .await?)
}

async fn get_or_create_transaction(
    connection: &mut Connection,
    user_name: &str,
    user_email: &str,
    date: &NaiveDateTime,
) -> Result<User, diesel::result::Error> {
    use schema::users::dsl::*;

    let user = users::table()
        .filter(email.eq(user_email))
        .first::<User>(connection)
        .await
        .optional();

    match user {
        Ok(None) => {
            // fall through.
        }
        Ok(Some(user)) => return Ok(user),
        Err(e) => return Err(e.into()),
    }

    let new_user = NewUser {
        name: user_name,
        email: user_email,
        creation_date: *date,
        last_login_date: *date,
    };

    new_user.insert_into(users).execute(connection).await?;

    let user = users::table()
        .filter(email.eq(user_email))
        .first::<User>(connection)
        .await?;

    Ok(user)
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use anyhow::Result;
    use diesel_async::{AsyncConnection, AsyncPgConnection};
    use rstest::*;
    use tokio_postgres::NoTls;

    use crate::test_helper::{self, postgres_container};

    #[rstest]
    #[tokio::test]
    async fn new_user(postgres_container: impl Future<Output = Result<String>>) -> Result<()> {
        let container = postgres_container.await?;

        println!("Connecting to container: {container}");
        // let connection = AsyncPgConnection::establish(&container).await?;

        let container = "postgres://armin:test@localhost:5432/stack-zero";

        let connection = tokio_postgres::connect(&container, NoTls).await?;

        Ok(())
    }
}
