use std::{net::SocketAddr, sync::Arc};

use ::anyhow::Result;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    response::Redirect,
    routing::get,
    Router,
};
use chrono::{DateTime, NaiveDateTime};
use chrono::{FixedOffset, Utc};
use derive_more::Constructor;
use dotenv::dotenv;
use jsonwebtoken as jwt;
use jwt::jwk::JwkSet;
use sea_orm::{prelude::DateTimeWithTimeZone, Database, DatabaseConnection};
use serde::Deserialize;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use tokio::net::TcpListener;
use url::Url;

mod anyhow;
mod auth0;
mod identity;
mod schema;
#[cfg(test)]
mod test_helper;
mod user;

use crate::id_token::IdToken;
use anyhow::AppError;
pub use identity::*;

#[derive(Constructor)]
struct Config {
    pub auth0: auth0::Config,
    pub jwk_set: JwkSet,
    pub db_connection: DatabaseConnection,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let auth0_config = auth0::Config::from_env()?;

    let jwks = auth0_config.download_jwk_set().await?;

    println!("jwk set: {:?}", jwks);

    let database = Database::connect(std::env::var("DATABASE_URL")?).await?;

    // let users = users::table
    //     .select(User::as_select())
    //     .load(&mut connection)
    //     .await?;

    // println!("Users: {:?}", users);

    let config = Arc::new(Config::new(auth0_config, jwks, database));

    let app = Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .with_state(config);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn login(State(state): State<Arc<Config>>) -> impl IntoResponse {
    // TODO: we can pre-create the full url in the configuration.

    let auth0 = &state.auth0;
    let base_url = format!("https://{}/authorize", auth0.domain);
    let mut url = Url::parse(&base_url).expect("Failed to parse URL");

    // TODO: add state.
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("response_type", "code");
        pairs.append_pair("client_id", &auth0.client_id);
        pairs.append_pair("redirect_uri", &auth0.callback_url);
        pairs.append_pair("scope", "openid profile email");
    }

    println!("redirecting to {url}");

    Redirect::temporary(url.as_str())
}

#[derive(Debug, Deserialize)]
struct AuthCallbackQuery {
    // TODO: Add state here.
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TokenResponse {
    Success {
        access_token: String,
        expires_in: usize,
        id_token: String,
        scope: String,
        token_type: String,
    },
    Error {
        error: TokenResponseError,
        error_description: String,
        error_uri: Option<String>,
    },
}

// <https://datatracker.ietf.org/doc/html/rfc6749#section-5.2>
// Well the error cods seem to be mixed up, so we added most of them here.
// TODO: Complete this list or find exactly out what errors are return in which contexts (5.2 does not include AccessDenied)
#[derive(Debug, Deserialize_enum_str, Serialize_enum_str, PartialEq)]
#[serde(rename_all = "snake_case")]
enum TokenResponseError {
    AccessDenied,
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
    #[serde(other)]
    Other(String),
}

/// This handle implements access token request as specified in:
///
/// <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.3>
/// <https://auth0.com/docs/api/authentication#authorization-code-flow47>

async fn callback(
    Query(query_params): Query<AuthCallbackQuery>,
    State(state): State<Arc<Config>>,
) -> Result<impl IntoResponse, AppError> {
    Ok(authorized(&query_params.code, &state).await?)
}

async fn authorized(authorization_code: &str, config: &Config) -> Result<impl IntoResponse> {
    let auth0 = &config.auth0;

    let token_response = reqwest::Client::new()
        .post(format!("https://{}/oauth/token", auth0.domain))
        .form(&[
            ("grant_type", "authorization_code"), // ("redirect_uri", "YOUR_CALLBACK_URI"),
            ("code", authorization_code),
            // required, and must be identical to the authorize/ request.
            ("redirect_uri", &auth0.callback_url),
            ("client_id", auth0.client_id.as_str()),
            ("client_secret", &auth0.client_secret),
        ])
        .send()
        .await?
        // TODO: May check for 404 response before parsing out errors?
        .json::<TokenResponse>()
        .await?;

    match &token_response {
        // TODO: should we check `scope`
        TokenResponse::Success { id_token, .. } => {
            let token = IdToken::validate(
                &(format!("https://{}/", config.auth0.domain)),
                &config.auth0.client_id,
                &config.jwk_set,
                id_token,
            )?;
            println!("Token successfully validated, inserting user");
            let connection = &config.db_connection;
            let claims = &token.claims;
            let user = crate::user::users::get_or_create(
                connection,
                &claims.profile.name,
                &claims.email.email,
                DateTime::<FixedOffset>::from(Utc::now()),
            )
            .await?;
            println!("User created with id: {}", user.id);
        }
        TokenResponse::Error {
            error,
            error_description,
            error_uri,
        } => {}
    }

    println!("{token_response:?}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            "\"test\"",
            serde_json::to_string(&TokenResponseError::Other("test".into())).unwrap()
        );

        assert_eq!(
            TokenResponseError::Other("test".into()),
            serde_json::from_str("\"test\"").unwrap()
        )
    }
}
