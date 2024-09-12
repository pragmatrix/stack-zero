pub use sea_orm_migration::prelude::*;

mod m20240823_000001_create_user_table;
mod m20240910_163755_add_user_password;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240823_000001_create_user_table::Migration),
            Box::new(m20240910_163755_add_user_password::Migration),
        ]
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
    Email,
    CreationDate,
    Password,
}
