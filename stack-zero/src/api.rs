use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use css_inline::CSSInliner;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::OpenApi;
use validator::Validate;

use crate::{email, AppError, StackZero};

#[derive(OpenApi)]
#[openapi(
    paths(sign_up),
    tags(
        (name = "stack-zero", description = "Stack Zero API")
    )
)]
pub struct Doc;

#[utoipa::path(
        get,
        path = "/sign-up",
        // params(
        //     ("id" = u64, Path, description = "Pet database id to get Pet for"),
        // ),
        // responses(
        //     (status = 200, description = "Pet found successfully", body = Pet),
        //     (status = NOT_FOUND, description = "Pet was not found")
        // ),
    )]

pub async fn sign_up(
    State(state): State<Arc<StackZero>>,
    Json(sign_up): Json<api::SignUp>,
) -> Result<Response, AppError> {
    sign_up.validate()?;

    println!("{sign_up:?}");

    let verification_link = email::verification::link(&state.config.base_url, &sign_up.email)?;

    let email = sign_up.email;

    let site = &state.config.base_url;

    let email = state.render(
        "emails/email_verification",
        json! {{"site": site, "name": email, "email": email}},
    )?;

    // TODO: cache inlining?

    let inliner = CSSInliner::default();
    let email = inliner.inline(&email)?;

    Ok((StatusCode::ACCEPTED, ()).into_response())
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum ApiResponse<T: Serialize> {
    Success { data: Option<T>, message: String },
    Error { error: String, details: String },
}

mod response {
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
        Json,
    };
    use serde_json::json;

    pub fn success(code: StatusCode, message: &str) -> Response {
        (
            code,
            Json(json! { {
                "message": message
            } }),
        )
            .into_response()
    }

    pub fn error(code: StatusCode, error: &str, details: &str) -> Response {
        (
            code,
            Json(json! { {
                "error": error,
                "details": details,
            } }),
        )
            .into_response()
    }
}
