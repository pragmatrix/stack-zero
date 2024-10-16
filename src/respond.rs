use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Serialize;

use crate::AppError;

pub fn html(content: &str) -> Result<Response, AppError> {
    Ok(Html(content.to_string()).into_response())
}

pub fn redirect(to: &str) -> Result<Response, AppError> {
    Ok(Redirect::to(to).into_response())
}
