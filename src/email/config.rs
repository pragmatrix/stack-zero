use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use url::Url;

const DEFAULT_SMTP_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Server and (optionally a port)
    server: String,
    credentials: ConnectionCredentials,
    from_address: Option<String>,
    timeout: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct ConnectionCredentials {
    username: String,
    password: String,
    security: ConnectionSecurity,
}

#[derive(Debug)]
pub struct EffectiveSmtp {
    server: String,
    port: u16,
    credentials: ConnectionCredentials,
    from_address: String,
    timeout: Duration,
}

impl Config {
    pub fn into_effective(self, site: &Url) -> Result<EffectiveSmtp> {
        let (server, port) = self.effective_server_and_port()?;
        let timeout = self.effective_timeout();
        let from_address = self.effective_from_address(site)?;

        Ok(EffectiveSmtp {
            server,
            port,
            credentials: self.credentials,
            from_address,
            timeout,
        })
    }

    pub fn effective_server_and_port(&self) -> Result<(String, u16)> {
        // Determine the default port based on encryption type
        let default_port = self.credentials.security.default_port();

        // Ensure the URL has a scheme; default to "smtp://" if missing
        let server_url = if self.server.contains("://") {
            self.server.clone()
        } else {
            format!("smtp://{}", self.server)
        };

        // Parse the URL
        let url = Url::parse(&server_url)?;
        let host = url
            .host_str()
            .ok_or(anyhow!("Invalid server URL"))?
            .to_string();

        // Use the specified port or fall back to the default port
        let port = url.port().unwrap_or(default_port);

        Ok((host, port))
    }

    pub fn effective_from_address(&self, site: &Url) -> Result<String> {
        // Check if the from_address is already specified
        if let Some(from_address) = &self.from_address {
            return Ok(from_address.clone());
        }

        // Generate the noreply email address based on the site URL
        let host = site
            .host_str()
            .ok_or(anyhow!("Can't resolve default from address"))?;
        let noreply_address = format!("noreply@{}", host);

        Ok(noreply_address)
    }

    pub fn effective_timeout(&self) -> Duration {
        self.timeout
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_SMTP_TIMEOUT)
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
enum ConnectionSecurity {
    #[default]
    Tls,
    StartTls,
    None,
}

impl ConnectionSecurity {
    pub fn default_port(&self) -> u16 {
        match self {
            ConnectionSecurity::None => 25,
            ConnectionSecurity::StartTls => 587,
            ConnectionSecurity::Tls => 465,
        }
    }
}
