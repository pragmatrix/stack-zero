use std::{collections::HashMap, env, future::Future};

use anyhow::{Context, Result};
use bollard::{
    container::{Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions},
    image::{self, CreateImageOptions},
    service::{HostConfig, PortBinding},
    Docker,
};
use dotenv::dotenv;
use futures_util::TryStreamExt;
use rstest::{fixture, rstest};
use sea_orm::Database;

#[rstest]
#[tokio::test]
async fn recreate_container_and_connect_to_db(
    postgres_container: impl Future<Output = Result<String>>,
) -> Result<()> {
    dotenv()?;

    let _container = postgres_container.await?;
    let _database = Database::connect(env::var("DATABASE_URL")?).await?;

    Ok(())
}

mod postgres {
    pub const IMAGE_NAME: &str = "postgres:16.4";
    pub const CONTAINER_NAME: &str = "stack-zero-postgres-test";
}
mod redis {
    pub const IMAGE_NAME: &str = "redis/redis-stack:7.4.0-v0";
    pub const CONTAINER_NAME: &str = "stack-zero-redis-test";
}

const POSTGRES_PASSWORD: &str = "stack-zero-test";
const POSTGRES_DB: &str = "stack-zero";

#[fixture]
pub async fn postgres_container() -> Result<String> {
    let docker = Docker::connect_with_local_defaults()?;

    // PostgreSQL configuration
    let user_var: &str = "POSTGRES_USER=postgres";
    let password_var: &str = &format!("POSTGRES_PASSWORD={POSTGRES_PASSWORD}");
    let db_var: &str = &format!("POSTGRES_DB={POSTGRES_DB}");
    let env_vars = vec![user_var, password_var, db_var];

    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        "5432/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("127.0.0.1".to_string()),
            host_port: Some("5433".to_string()),
        }]),
    );

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        ..Default::default()
    };

    let image_name = postgres::IMAGE_NAME;

    pull_image(&docker, image_name).await?;

    let config = Config {
        image: Some(image_name),
        env: Some(env_vars),
        host_config: Some(host_config),
        ..Default::default()
    };

    let container_name = postgres::CONTAINER_NAME;

    let create_options = CreateContainerOptions {
        name: container_name.to_string(),
        ..Default::default()
    };

    recreate_container(&docker, create_options, config)
        .await
        .context("Recreating container")?;

    docker
        .start_container(container_name, None::<StartContainerOptions<String>>)
        .await?;

    Ok(format!(
        "postgres://postgres:{POSTGRES_PASSWORD}@localhost:5433/stack-zero"
    )) // Connection string (adjust as needed)
}

/// Redis container for session management.
#[fixture]
pub async fn redis_container() -> Result<String> {
    let docker = Docker::connect_with_local_defaults()?;

    let env_vars = vec![];

    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        "6379/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("127.0.0.1".to_string()), // Bind to all interfaces
            host_port: Some("6379".to_string()),    // Map to the same port on the host
        }]),
    );

    port_bindings.insert(
        "8001/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("127.0.0.1".to_string()), // Bind to all interfaces
            host_port: Some("8001".to_string()),    // Map to the same port on the host
        }]),
    );

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        ..Default::default()
    };

    let image_name = redis::IMAGE_NAME;

    pull_image(&docker, image_name).await?;

    let config = Config {
        image: Some(image_name),
        env: Some(env_vars),
        host_config: Some(host_config),
        ..Default::default()
    };

    let container_name = redis::CONTAINER_NAME;

    let create_options = CreateContainerOptions {
        name: container_name.to_string(),
        ..Default::default()
    };

    recreate_container(&docker, create_options, config)
        .await
        .context("Recreating container")?;

    docker
        .start_container(container_name, None::<StartContainerOptions<String>>)
        .await?;

    Ok("".into())
}

async fn recreate_container(
    docker: &Docker,
    create_options: CreateContainerOptions<String>,
    config: bollard::container::Config<&str>,
) -> Result<()> {
    // Check if the container exists
    let containers = docker
        .list_containers::<String>(None)
        .await
        .context("list containers")?;
    let container_name = &create_options.name;
    let name = &format!("/{}", container_name);
    if containers
        .iter()
        .flat_map(|container| &container.names)
        .any(|names| names.contains(name))
    {
        docker
            .stop_container(container_name, None)
            .await
            .context("Stopping container")?;

        // If it does, remove it
        docker
            .remove_container(container_name, None::<RemoveContainerOptions>)
            .await
            .context("Removing container")?;
    }

    // Create the new container
    docker
        .create_container(Some(create_options), config)
        .await
        .context("Creating container")?;

    Ok(())
}

async fn pull_image(docker: &Docker, image_name: &str) -> Result<()> {
    let options = CreateImageOptions {
        from_image: image_name,
        ..CreateImageOptions::default()
    };

    docker
        .create_image(Some(options), None, None)
        .try_collect::<Vec<_>>()
        .await?;

    Ok(())
}
