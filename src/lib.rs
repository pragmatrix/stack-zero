use std::{default, env, fs, path::PathBuf, sync::Arc};

use ::anyhow::{Context, Result};
use axum::{
    extract::{FromRef, Query, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use env_renderer::EnvRenderer;
use jsonwebtoken as jwt;
use jwt::jwk::JwkSet;
use sea_orm::{Database, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use session::SessionStore;
use tokio::task::JoinHandle;
use tower_http::services::ServeDir;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use url::Url;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::id_token::IdToken;
use user::users;
use view_renderer::*;

mod anyhow;
mod api;
mod auth0;
mod config;
mod email;
mod env_renderer;
mod identity;
pub mod respond;
mod session;
#[cfg(test)]
mod test_helper;
mod user;
mod view_renderer;

pub use anyhow::AppError;
pub use identity::*;

#[derive(Debug)]
pub struct StackZero {
    pub config: Config,
    pub smtp_config: email::Config,
    pub auth0: auth0::Config,
    pub jwk_set: JwkSet,
    pub session_store: SessionStore,
    pub db_connection: DatabaseConnection,
    pub template_renderer: ViewRenderer,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum Environment {
    #[default]
    Development,
    Production,
}

impl Environment {
    /// Cookies should be usable only over HTTPS?
    pub fn use_secure_cookies(&self) -> bool {
        match self {
            Environment::Development => false,
            Environment::Production => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    /// Used for example in Email verification link generation.
    pub base_url: Url,
    pub config_file: PathBuf,
    // TODO: read this from the config file.
    pub template_dir: PathBuf,
    pub environment: Environment,
}

impl Config {
    pub fn from_base_url(url: Url) -> Self {
        Self {
            base_url: url,
            config_file: "stack-zero.toml".into(),
            template_dir: "assets".into(),
            environment: Environment::default(),
        }
    }
}

impl StackZero {
    pub async fn new(config: Config) -> Result<Self> {
        let env_renderer = EnvRenderer::from_env();

        let stack_zero_conf: config::Config = {
            let file = &config.config_file;
            let toml = fs::read_to_string(file)
                .with_context(|| format!("Failed to read configuration file from {:?}", file))?;
            let with_env = env_renderer.render(&toml)?;
            toml::from_str(&with_env)?
        };

        let template_renderer = ViewRenderer::from_dir(&config.template_dir)?;

        // TODO: load auth0 config from stack-zero.conf

        let auth0 = auth0::Config::from_env()?;
        let jwk_set = auth0.download_jwk_set().await?;

        println!("jwk set: {:?}", jwk_set);

        let database = Database::connect(env::var("DATABASE_URL")?).await?;

        let session_store = SessionStore::from_env(config.environment).await?;

        Ok(Self {
            config,
            smtp_config: stack_zero_conf.smtp,
            auth0,
            jwk_set,
            session_store,
            db_connection: database,
            template_renderer,
        })
    }

    pub fn install_routes<State>(&self, router: Router<State>) -> Router<State>
    where
        Arc<StackZero>: FromRef<State>,
        State: Clone + Send + Sync + 'static,
    {
        let static_files_service = ServeDir::new("assets/static");

        // let session_store = MemoryStore::default();

        let router = router
            .route("/login", get(login))
            .route("/callback", get(callback))
            .nest_service("/static", static_files_service)
            .route("/api/sign-up", post(api::sign_up))
            .merge(Scalar::with_url("/api", api::Doc::openapi()));

        self.session_store
            .add_layer(self.config.environment, router)
    }

    pub fn render(&self, key: &str, data: impl Serialize) -> Result<String> {
        self.template_renderer.render(key, data)
    }

    pub async fn users(&self) -> Result<Vec<entity::user::Model>> {
        Ok(entity::user::Entity::find()
            .all(&self.db_connection)
            .await?)
    }
}

async fn login(State(state): State<Arc<StackZero>>) -> impl IntoResponse {
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
    State(state): State<Arc<StackZero>>,
) -> Result<impl IntoResponse, AppError> {
    Ok(authorized(&query_params.code, &state).await?)
}

async fn authorized(authorization_code: &str, config: &StackZero) -> Result<impl IntoResponse> {
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
            let user = users::create(
                connection,
                &claims.profile.name,
                &claims.email.email,
                users::AuthenticationMethod::SingleSignOn,
                Utc::now().into(),
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
