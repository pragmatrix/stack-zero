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

    Ok(new_user)
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use anyhow::Result;
    use rstest::*;
    use tokio_postgres::NoTls;

    use crate::test_helper::postgres_container;

    #[rstest]
    #[tokio::test]
    async fn new_user(postgres_container: impl Future<Output = Result<String>>) -> Result<()> {
        let container = postgres_container.await?;

        println!("Connecting to container: {container}");
        // let connection = AsyncPgConnection::establish(&container).await?;

        let container = "postgres://armin:test@localhost:5432/stack-zero";

        let _connection = tokio_postgres::connect(container, NoTls).await?;

        todo!("Create a new user");

        // Ok(())
    }
}
