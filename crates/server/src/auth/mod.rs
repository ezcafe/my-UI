pub mod crypto;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod oidc;
pub mod session;
pub mod temp_cookie;

pub use middleware::AuthenticatedUser;

#[cfg(test)]
mod tests {
    use super::temp_cookie::{
        create_temp_cookie_value, OidcTempData,
    };
    use crate::app::build_app;
    use crate::config::Config;
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use miniflux_client::client::MinifluxClient;
    use miniflux_client::schema::build_schema;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_config() -> Arc<Config> {
        Arc::new(Config {
            host: "127.0.0.1".to_string(),
            port: 3000,
            app_base_url: "http://127.0.0.1:3000".to_string(),
            session_secret: "super-secret-key-that-is-at-least-32-bytes-long!".to_string(),
            oidc_issuer_url: "https://id.example.com".to_string(),
            oidc_client_id: "test-client".to_string(),
            oidc_client_secret: "test-secret".to_string(),
            oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
            miniflux_base_url: "https://miniflux.example.com".to_string(),
            miniflux_api_key: "test-api-key".to_string(),
            ai_enabled: false,
            llm_cli_cmd: "llm".to_string(),
            llm_model: "gpt-4o-mini".to_string(),
            llm_extra_args: vec![],
            ai_db_path: "data/summaries.db".to_string(),
        })
    }

    #[tokio::test]
    async fn test_unauthenticated_request_redirects_to_login() {
        let config = test_config();
        let client = MinifluxClient::new(config.miniflux_base_url.clone(), config.miniflux_api_key.clone());
        let schema = build_schema(client);
        let app = build_app(config, schema);

        let req = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        assert_eq!(resp.headers().get(header::LOCATION).unwrap(), "/login");
    }

    #[tokio::test]
    async fn test_htmx_unauthenticated_request_returns_hx_redirect() {
        let config = test_config();
        let client = MinifluxClient::new(config.miniflux_base_url.clone(), config.miniflux_api_key.clone());
        let schema = build_schema(client);
        let app = build_app(config, schema);

        let req = Request::builder()
            .uri("/hx/entries")
            .header("HX-Request", "true")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get("HX-Redirect").unwrap(), "/login");
    }

    #[tokio::test]
    async fn test_oidc_callback_state_mismatch_fails() {
        let config = test_config();
        let client = MinifluxClient::new(config.miniflux_base_url.clone(), config.miniflux_api_key.clone());
        let schema = build_schema(client);
        let app = build_app(config.clone(), schema);

        let temp_data = OidcTempData {
            pkce_verifier: "verifier123".to_string(),
            csrf_state: "expected_state_abc".to_string(),
        };
        let encrypted = create_temp_cookie_value(&temp_data, &config.session_secret).unwrap();

        let req = Request::builder()
            .uri("/auth/callback?code=valid_code&state=wrong_state")
            .header(header::COOKIE, format!("__oidc_temp={}", encrypted))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_oidc_callback_clears_temp_cookie() {
        let config = test_config();
        let client = MinifluxClient::new(config.miniflux_base_url.clone(), config.miniflux_api_key.clone());
        let schema = build_schema(client);
        let app = build_app(config.clone(), schema);

        let temp_data = OidcTempData {
            pkce_verifier: "verifier123".to_string(),
            csrf_state: "matching_state".to_string(),
        };
        let encrypted = create_temp_cookie_value(&temp_data, &config.session_secret).unwrap();

        let req = Request::builder()
            .uri("/auth/callback?code=valid_code&state=matching_state")
            .header(header::COOKIE, format!("__oidc_temp={}", encrypted))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        assert_eq!(resp.headers().get(header::LOCATION).unwrap(), "/");

        let cookies: Vec<&str> = resp
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|h| h.to_str().unwrap())
            .collect();

        assert!(cookies.iter().any(|c| c.contains("__session=")));
        assert!(cookies.iter().any(|c| c.contains("__oidc_temp=") && c.contains("Max-Age=0")));
    }

    #[tokio::test]
    async fn test_unauthenticated_graphql_request_rejected_with_401() {
        let config = test_config();
        let client = MinifluxClient::new(config.miniflux_base_url.clone(), config.miniflux_api_key.clone());
        let schema = build_schema(client);
        let app = build_app(config, schema);

        let req = Request::builder()
            .method("POST")
            .uri("/graphql")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"query": "{ feeds { id } }"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_expired_session_rejected() {
        use super::session::{create_session_cookie_value, SessionData};

        let config = test_config();
        let client = MinifluxClient::new(config.miniflux_base_url.clone(), config.miniflux_api_key.clone());
        let schema = build_schema(client);
        let app = build_app(config.clone(), schema);

        let expired_session = SessionData {
            user_id: "usr_exp".to_string(),
            username: "expired_user".to_string(),
            access_token: "expired_token".to_string(),
            refresh_token: None,
            expires_at: chrono::Utc::now().timestamp() - 300, // 5 minutes in the past
        };
        let encrypted = create_session_cookie_value(&expired_session, &config.session_secret).unwrap();

        // 1. Regular browser GET to / -> should redirect to /login
        let req = Request::builder()
            .uri("/")
            .header(header::COOKIE, format!("__session={}", encrypted))
            .body(Body::empty())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        assert_eq!(resp.headers().get(header::LOCATION).unwrap(), "/login");

        // 2. HTMX request -> should return HX-Redirect
        let req_htmx = Request::builder()
            .uri("/hx/entries")
            .header("HX-Request", "true")
            .header(header::COOKIE, format!("__session={}", encrypted))
            .body(Body::empty())
            .unwrap();

        let resp_htmx = app.clone().oneshot(req_htmx).await.unwrap();
        assert_eq!(resp_htmx.status(), StatusCode::OK);
        assert_eq!(resp_htmx.headers().get("HX-Redirect").unwrap(), "/login");

        // 3. API / GraphQL request -> should return 401
        let req_gql = Request::builder()
            .method("POST")
            .uri("/graphql")
            .header(header::COOKIE, format!("__session={}", encrypted))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"query": "{ feeds { id } }"}"#))
            .unwrap();

        let resp_gql = app.oneshot(req_gql).await.unwrap();
        assert_eq!(resp_gql.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_secure_cookie_flag_when_https() {
        let mut cfg = (*test_config()).clone();
        cfg.app_base_url = "https://app.example.com".to_string();
        cfg.host = "app.example.com".to_string();
        let config = Arc::new(cfg);

        let client = MinifluxClient::new(config.miniflux_base_url.clone(), config.miniflux_api_key.clone());
        let schema = build_schema(client);
        let app = build_app(config.clone(), schema);

        let temp_data = OidcTempData {
            pkce_verifier: "verifier123".to_string(),
            csrf_state: "matching_state".to_string(),
        };
        let encrypted = create_temp_cookie_value(&temp_data, &config.session_secret).unwrap();

        let req = Request::builder()
            .uri("/auth/callback?code=valid_code&state=matching_state")
            .header(header::COOKIE, format!("__oidc_temp={}", encrypted))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);

        let cookies: Vec<&str> = resp
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|h| h.to_str().unwrap())
            .collect();

        for c in &cookies {
            assert!(c.contains("Secure"), "Cookie must contain Secure flag in HTTPS: {}", c);
        }
    }
}
