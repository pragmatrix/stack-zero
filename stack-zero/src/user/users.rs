use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use entity::user;
use sea_orm::{prelude::*, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};

pub async fn get_or_create(
    connection: &DatabaseConnection,
    user_name: &str,
    user_email: &str,
    date: DateTime<FixedOffset>,
) -> Result<user::Model> {
    let txn = connection.begin().await?;

    let user = user::Entity::find()
        .filter(user::Column::Email.eq(user_email))
        .one(&txn)
        .await?;

    if let Some(user) = user {
        return Ok(user);
    }

    let new_user = user::Model {
        id: Uuid::new_v4(),
        name: user_name.into(),
        email: user_email.into(),
        creation_date: date,
    };

    user::Entity::insert(user::ActiveModel::from(new_user.clone()))
        .exec(&txn)
        .await?;

    txn.commit().await?;

    Ok(new_user)
}

#[cfg(test)]
mod tests {
    use std::{env, future::Future};

    use anyhow::Result;
    use chrono::{DateTime, FixedOffset, Utc};
    use dotenv::dotenv;
    use rstest::*;
    use sea_orm::Database;

    use crate::test_helper::postgres_container;
    #[rstest]
    #[tokio::test]
    #[ignore = "manually only"]
    async fn new_user(postgres_container: impl Future<Output = Result<String>>) -> Result<()> {
        dotenv()?;

        let container = postgres_container.await?;
        println!("Connecting to container: {container}");

        // let container = "postgres://armin:test@localhost:5432/stack-zero";

        let _database = Database::connect(env::var("DATABASE_URL")?).await?;

        let user =
            super::get_or_create(&_database, "John Doe", "john@doe.com", Utc::now().into()).await?;

        println!("User in db: {user:?}");

        Ok(())
    }
}
