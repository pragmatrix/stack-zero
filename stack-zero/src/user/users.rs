use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use entity::user;
use sea_orm::{prelude::*, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};

#[derive(Debug, Clone)]
pub enum AuthenticationMethod {
    SingleSignOn,
    Password(String),
}

/// Create a new user.
pub async fn create(
    connection: &DatabaseConnection,
    name: &str,
    email: &str,
    authentication_method: AuthenticationMethod,
    date: DateTime<FixedOffset>,
) -> Result<user::Model> {
    let password = match authentication_method {
        AuthenticationMethod::SingleSignOn => "".into(),
        AuthenticationMethod::Password(pw) => password_auth::generate_hash(&pw),
    };

    let new_user = user::Model {
        id: Uuid::new_v4(),
        name: name.into(),
        email: email.into(),
        creation_date: date,
        password,
    };

    {
        let txn = connection.begin().await?;

        user::Entity::insert(user::ActiveModel::from(new_user.clone()))
            .exec(&txn)
            .await?;

        txn.commit().await?;
    }

    Ok(new_user)
}

pub async fn get_by_email(
    connection: &DatabaseConnection,
    user_email: &str,
) -> Result<Option<user::Model>> {
    Ok(user::Entity::find()
        .filter(user::Column::Email.eq(user_email))
        .one(connection)
        .await?)
}

#[cfg(test)]
mod tests {
    use std::{env, future::Future};

    use anyhow::Result;
    use chrono::Utc;
    use dotenv::dotenv;
    use rstest::*;
    use sea_orm::Database;

    use crate::test_helper::postgres_container;

    use super::AuthenticationMethod;

    #[rstest]
    #[tokio::test]
    #[ignore = "manually only"]
    async fn new_user(postgres_container: impl Future<Output = Result<String>>) -> Result<()> {
        dotenv()?;

        let container = postgres_container.await?;
        println!("Connecting to container: {container}");

        let _database = Database::connect(env::var("DATABASE_URL")?).await?;

        let user = super::create(
            &_database,
            "John Doe",
            "john@doe.com",
            AuthenticationMethod::SingleSignOn,
            Utc::now().into(),
        )
        .await?;

        println!("User in db: {user:?}");

        Ok(())
    }
}
