use super::middleware::extract_cookie;
use super::oidc::{exchange_token, generate_auth_request};
use super::session::{
    build_clear_session_cookie_header, build_session_set_cookie_header, create_session_cookie_value,
};
use super::temp_cookie::{
    build_clear_temp_cookie_header, build_temp_set_cookie_header, create_temp_cookie_value,
    extract_temp_cookie_value, OidcTempData, OIDC_TEMP_COOKIE_NAME,
};
use crate::config::Config;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct AuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

pub async fn login_handler(
    State(config): State<Arc<Config>>,
) -> Result<Response, (StatusCode, String)> {
    if !config.auth_enabled {
        return Ok(Redirect::to("/").into_response());
    }

    let req = generate_auth_request(&config)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let temp_data = OidcTempData {
        pkce_verifier: req.pkce_verifier,
        csrf_state: req.csrf_state,
    };

    let encrypted_temp = create_temp_cookie_value(&temp_data, &config.session_secret)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let cookie_header = build_temp_set_cookie_header(&encrypted_temp, config.is_secure());

    let mut response = Redirect::to(&req.authorize_url).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie_header.parse().unwrap());
    Ok(response)
}

pub async fn callback_handler(
    State(config): State<Arc<Config>>,
    headers: HeaderMap,
    Query(query): Query<AuthCallbackQuery>,
) -> Result<Response, (StatusCode, String)> {
    if !config.auth_enabled {
        return Ok(Redirect::to("/").into_response());
    }

    if let Some(err) = query.error {
        return Err((StatusCode::BAD_REQUEST, format!("OIDC error: {}", err)));
    }

    let code = query
        .code
        .ok_or((StatusCode::BAD_REQUEST, "Missing code".to_string()))?;
    let state = query
        .state
        .ok_or((StatusCode::BAD_REQUEST, "Missing state".to_string()))?;

    let temp_cookie_val = extract_cookie(&headers, OIDC_TEMP_COOKIE_NAME).ok_or((
        StatusCode::UNAUTHORIZED,
        "Missing temp OIDC cookie".to_string(),
    ))?;

    let temp_data =
        extract_temp_cookie_value(temp_cookie_val, &config.session_secret).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid temp OIDC cookie".to_string(),
            )
        })?;

    if temp_data.csrf_state != state {
        return Err((StatusCode::BAD_REQUEST, "CSRF state mismatch".to_string()));
    }

    // Exchange token: in integration / production, dispatches token request with pkce_verifier
    let session = exchange_token(&config, &code, &temp_data.pkce_verifier)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Token exchange failed: {}", e),
            )
        })?;

    let encrypted_session = create_session_cookie_value(&session, &config.session_secret)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session_cookie = build_session_set_cookie_header(&encrypted_session, config.is_secure());
    let clear_temp_cookie = build_clear_temp_cookie_header(config.is_secure());

    let mut response = Redirect::to("/").into_response();
    let resp_headers = response.headers_mut();
    resp_headers.append(header::SET_COOKIE, session_cookie.parse().unwrap());
    resp_headers.append(header::SET_COOKIE, clear_temp_cookie.parse().unwrap());

    Ok(response)
}

pub async fn logout_handler(State(config): State<Arc<Config>>) -> Response {
    if !config.auth_enabled {
        return Redirect::to("/").into_response();
    }

    let clear_session = build_clear_session_cookie_header(config.is_secure());
    let mut response = Redirect::to("/login").into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, clear_session.parse().unwrap());
    response
}
