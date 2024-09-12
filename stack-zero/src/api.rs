use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;

use crate::{AppError, StackZero};

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
    println!("{sign_up:?}");
    todo!();
    Ok((StatusCode::CREATED, ()).into_response())
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
