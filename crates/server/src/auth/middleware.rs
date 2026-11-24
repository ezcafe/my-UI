use super::session::{extract_session_cookie_value, SessionData, SESSION_COOKIE_NAME};
use crate::config::Config;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub username: String,
    pub access_token: String,
}

pub struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        Redirect::to("/login").into_response()
    }
}

pub fn extract_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|s| {
            let (key, val) = s.trim().split_once('=')?;
            if key == name {
                Some(val)
            } else {
                None
            }
        })
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    Arc<Config>: axum::extract::FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Arc::<Config>::from_ref(state);
        if !config.auth_enabled {
            return Ok(AuthenticatedUser {
                user_id: "guest".to_string(),
                username: "Guest".to_string(),
                access_token: String::new(),
            });
        }

        let is_htmx = parts
            .headers
            .get("HX-Request")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "true")
            .unwrap_or(false);

        let is_api = parts.uri.path().starts_with("/graphql")
            || parts
                .headers
                .get(header::ACCEPT)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.contains("application/json"))
                .unwrap_or(false);

        let unauth_response = || {
            if is_htmx {
                let mut resp = StatusCode::OK.into_response();
                resp.headers_mut()
                    .insert("HX-Redirect", "/login".parse().unwrap());
                resp
            } else if is_api {
                StatusCode::UNAUTHORIZED.into_response()
            } else {
                Redirect::to("/login").into_response()
            }
        };

        let cookie_val = match extract_cookie(&parts.headers, SESSION_COOKIE_NAME) {
            Some(v) => v,
            None => return Err(unauth_response()),
        };

        let session: SessionData =
            match extract_session_cookie_value(cookie_val, &config.session_secret) {
                Ok(s) => s,
                Err(_) => return Err(unauth_response()),
            };

        let now = chrono::Utc::now().timestamp();
        if session.expires_at <= now {
            return Err(unauth_response());
        }

        Ok(AuthenticatedUser {
            user_id: session.user_id,
            username: session.username,
            access_token: session.access_token,
        })
    }
}
