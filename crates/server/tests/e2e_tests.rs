use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use miniflux_client::client::MinifluxClient;
use miniflux_client::schema::build_schema;
use serde_json::json;
use server::app::build_app;
use server::auth::session::{create_session_cookie_value, SessionData};
use server::config::Config;
use std::sync::Arc;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_app_starts_and_responds_to_health_check() {
    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        app_base_url: "http://127.0.0.1:3000".to_string(),
        session_secret: "super-secret-key-that-is-at-least-32-bytes-long!".to_string(),
        oidc_issuer_url: "https://id.example.com".to_string(),
        oidc_client_id: "client-id".to_string(),
        oidc_client_secret: "client-secret".to_string(),
        oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
        miniflux_base_url: "https://miniflux.example.com".to_string(),
        miniflux_api_key: "api-key".to_string(),
        ai_enabled: false,
        llm_cli_cmd: "llm".to_string(),
        llm_model: "gpt-4o-mini".to_string(),
        llm_extra_args: vec![],
        ai_db_path: "data/summaries.db".to_string(),
    });

    let client = MinifluxClient::new(config.miniflux_base_url.clone(), config.miniflux_api_key.clone());
    let schema = build_schema(client);
    let app = build_app(config, schema);

    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["status"], "ok");
}

#[tokio::test]
async fn test_e2e_authenticated_session_flow() {
    let mock_upstream = MockServer::start().await;

    // Upstream mocks
    Mock::given(method("GET"))
        .and(path("/v1/feeds/counters"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "reads": {},
            "unreads": { "1": 1 }
        })))
        .mount(&mock_upstream)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1/entries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total": 1,
            "entries": [
                {
                    "id": 999,
                    "feed_id": 1,
                    "title": "E2E Test Entry",
                    "url": "https://example.com/e2e",
                    "content": "Full content of the e2e test article.",
                    "published_at": "Today",
                    "status": "unread",
                    "starred": false,
                    "reading_time": 1
                }
            ]
        })))
        .mount(&mock_upstream)
        .await;

    Mock::given(method("PUT"))
        .and(path("/v1/entries"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_upstream)
        .await;

    let secret = "super-secret-key-that-is-at-least-32-bytes-long!";
    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        app_base_url: "http://127.0.0.1:3000".to_string(),
        session_secret: secret.to_string(),
        oidc_issuer_url: "https://id.example.com".to_string(),
        oidc_client_id: "client-id".to_string(),
        oidc_client_secret: "client-secret".to_string(),
        oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
        miniflux_base_url: mock_upstream.uri(),
        miniflux_api_key: "api-key".to_string(),
        ai_enabled: false,
        llm_cli_cmd: "llm".to_string(),
        llm_model: "gpt-4o-mini".to_string(),
        llm_extra_args: vec![],
        ai_db_path: "data/summaries.db".to_string(),
    });

    let client = MinifluxClient::new(mock_upstream.uri(), "api-key".to_string());
    let schema = build_schema(client);
    let app = build_app(config, schema);

    // 1. Establish session
    let session = SessionData {
        user_id: "usr_e2e".to_string(),
        username: "e2e_user".to_string(),
        access_token: "token_e2e".to_string(),
        refresh_token: None,
        expires_at: 1800000000,
    };
    let enc_session = create_session_cookie_value(&session, secret).unwrap();
    let session_cookie = format!("__session={}", enc_session);

    // 2. Fetch entry list
    let list_req = Request::builder()
        .uri("/hx/entries")
        .header("HX-Request", "true")
        .header(header::COOKIE, &session_cookie)
        .body(Body::empty())
        .unwrap();

    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body = String::from_utf8(list_resp.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap();
    assert!(list_body.contains("E2E Test Entry"));

    // 3. Mark entry as read
    let read_req = Request::builder()
        .method("POST")
        .uri("/hx/entries/999/read?status=read")
        .header("HX-Request", "true")
        .header(header::COOKIE, &session_cookie)
        .body(Body::empty())
        .unwrap();

    let read_resp = app.clone().oneshot(read_req).await.unwrap();
    assert_eq!(read_resp.status(), StatusCode::OK);
    let read_body = String::from_utf8(read_resp.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap();
    assert!(read_body.contains("id=\"entry-999\""));
    assert!(read_body.contains("id=\"unread-pill\""));

    // 4. Logout
    let logout_req = Request::builder()
        .uri("/auth/logout")
        .body(Body::empty())
        .unwrap();

    let logout_resp = app.oneshot(logout_req).await.unwrap();
    assert_eq!(logout_resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(logout_resp.headers().get(header::LOCATION).unwrap(), "/login");
    let set_cookie = logout_resp.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(set_cookie.contains("__session=") && set_cookie.contains("Max-Age=0"));
}
