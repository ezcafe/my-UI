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

impl AppError {
    pub fn user_facing_message(&self) -> &'static str {
        match self {
            AppError::NotFound(msg) if msg == "AI digest is disabled" => "AI digest is disabled.",
            AppError::NotFound(_) => "The requested item could not be found.",
            AppError::BadRequest(_) => "Invalid request. Please try again.",
            AppError::Upstream(_) => "Unable to connect to news service. Please try again later.",
            AppError::Internal(_) => "An unexpected error occurred. Please try again.",
        }
    }
}

pub fn handle_htmx_error(headers: &HeaderMap, error: AppError) -> Response {
    eprintln!("Application error: {:?}", error);

    let is_htmx = headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false);

    if is_htmx {
        let toast = ToastTemplate {
            message: error.user_facing_message().to_string(),
            is_error: true,
        };
        let toast_html = toast.render().unwrap_or_default();
        let mut resp = Html(toast_html).into_response();
        resp.headers_mut()
            .insert("HX-Reswap", "none".parse().unwrap());
        resp
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, error.user_facing_message().to_string()).into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_facing_error_messages_do_not_leak_internal_details() {
        let err_upstream = AppError::Upstream("Raw hyper connect error 127.0.0.1:56679".to_string());
        assert_eq!(
            err_upstream.user_facing_message(),
            "Unable to connect to news service. Please try again later."
        );
        assert!(!err_upstream.user_facing_message().contains("127.0.0.1"));

        let err_internal = AppError::Internal("Database connection pool poisoned".to_string());
        assert_eq!(
            err_internal.user_facing_message(),
            "An unexpected error occurred. Please try again."
        );
        assert!(!err_internal.user_facing_message().contains("Database"));

        let err_not_found = AppError::NotFound("entry 99999".to_string());
        assert_eq!(
            err_not_found.user_facing_message(),
            "The requested item could not be found."
        );

        let err_bad_req = AppError::BadRequest("invalid offset -1".to_string());
        assert_eq!(
            err_bad_req.user_facing_message(),
            "Invalid request. Please try again."
        );
    }

    #[test]
    fn test_handle_htmx_error_returns_clean_toast_without_debug_dump() {
        let mut headers = HeaderMap::new();
        headers.insert("HX-Request", "true".parse().unwrap());

        let resp = handle_htmx_error(&headers, AppError::Upstream("Panic at reqwest line 42".to_string()));
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get("HX-Reswap").unwrap(), "none");
    }
}
