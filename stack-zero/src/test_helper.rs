use std::{collections::HashMap, default};

use anyhow::{Context, Result};
use bollard::{
    container::{
        self, Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
    },
    service::{HostConfig, PortBinding, PortMap},
    Docker,
};
use rstest::fixture;

const IMAGE_NAME: &str = "postgres:16.1";
const CONTAINER_NAME: &str = "stack-zero-postgres-test";
const POSTGRES_PASSWORD: &str = "stack-zero-test";
const POSTGRES_DB: &str = "stack-zero";

#[fixture]
pub async fn postgres_container() -> Result<String> {
    let docker = Docker::connect_with_local_defaults()?;

    // PostgreSQL configuration
    let user_var: &str = &format!("POSTGRES_USER=postgres");
    let password_var: &str = &format!("POSTGRES_PASSWORD={POSTGRES_PASSWORD}");
    let db_var: &str = &format!("POSTGRES_DB={POSTGRES_DB}");
    let env_vars = vec![user_var, password_var, db_var];

    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        "5432/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("127.0.0.1".to_string()), // Bind to all interfaces
            host_port: Some("5433".to_string()),    // Map to the same port on the host
        }]),
    );

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        ..Default::default()
    };

    let config = Config {
        image: Some(IMAGE_NAME),
        env: Some(env_vars),
        host_config: Some(host_config),
        ..Default::default()
    };

    let container_name = CONTAINER_NAME;

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
