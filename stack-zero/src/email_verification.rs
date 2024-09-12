use std::{
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use url::Url;

// TODO: Use a toml file for configuration?
const EMAIL_VERIFICATION_EXPIRATION: Duration = Duration::from_secs(15 * 60);

// Define the claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    email: String,
    exp: u64,
}

fn generate_email_verification_link(endpoint: &Url, email: &str) -> Result<Url> {
    let jwt_secret = env::var("JWT_SECRET").context("JWT_SECRET not set")?;
    let jwt = generate_jwt(email, &jwt_secret)?;
    Ok(endpoint.join(&format!("&t={}", jwt))?)
}

fn generate_jwt(email: &str, secret_base64: &str) -> Result<String> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
        + EMAIL_VERIFICATION_EXPIRATION.as_secs();

    let claims = Claims {
        email: email.into(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_base64_secret(secret_base64)?,
    )?;

    Ok(token)
}
