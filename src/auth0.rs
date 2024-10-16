use std::env;

use anyhow::{Context, Result};
use jsonwebtoken::jwk;

#[derive(Debug)]
pub struct Config {
    pub domain: String,
    pub callback_url: String,
    pub client_id: String,
    pub client_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let client_id = env::var("AUTH0_CLIENT_ID").context("AUTH0_CLIENT_ID not set")?;
        let client_secret =
            env::var("AUTH0_CLIENT_SECRET").context("AUTH0_CLIENT_SECRET not set")?;
        let domain = env::var("AUTH0_DOMAIN").context("AUTH0_DOMAIN not set")?;
        let callback_url = env::var("AUTH0_CALLBACK_URL").context("AUTH0_CALLBACK_URL not set")?;

        Ok(Self {
            domain,
            callback_url,
            client_id,
            client_secret,
        })
    }

    /// Downloads the JWK set from the auth0 domain.
    pub async fn download_jwk_set(&self) -> Result<jwk::JwkSet> {
        let url = format!("https://{}/.well-known/jwks.json", &self.domain);
        Ok(reqwest::get(url).await?.json::<jwk::JwkSet>().await?)
    }
}
