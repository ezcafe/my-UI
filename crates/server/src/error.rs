use askama::Template;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use thiserror::Error;
use ui_components::templates::ToastTemplate;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Upstream error: {0}")]
    Upstream(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

pub fn handle_htmx_error(headers: &HeaderMap, error: AppError) -> Response {
    let is_htmx = headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false);

    if is_htmx {
        let toast = ToastTemplate {
            message: error.to_string(),
            is_error: true,
        };
        let toast_html = toast.render().unwrap_or_default();
        let mut resp = Html(toast_html).into_response();
        resp.headers_mut()
            .insert("HX-Reswap", "none".parse().unwrap());
        resp
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
