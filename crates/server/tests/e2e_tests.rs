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
        auth_enabled: true,
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
        database_url: ":memory:".to_string(),
        llm_provider: "openai".to_string(),
        llm_api_key: None,
        llm_api_base_url: None,
    });

    let client = MinifluxClient::new(
        config.miniflux_base_url.clone(),
        config.miniflux_api_key.clone(),
    );
    let schema = build_schema(client);
    let app = build_app(config, schema).await;

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
        auth_enabled: true,
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
        database_url: ":memory:".to_string(),
        llm_provider: "openai".to_string(),
        llm_api_key: None,
        llm_api_base_url: None,
    });

    let client = MinifluxClient::new(mock_upstream.uri(), "api-key".to_string());
    let schema = build_schema(client);
    let app = build_app(config, schema).await;

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
    let list_body = String::from_utf8(
        list_resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
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
    let read_body = String::from_utf8(
        read_resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(read_body.contains("id=\"entry-999\""));
    assert!(read_body.contains("id=\"unread-pill\""));

    // 4. Logout
    let logout_req = Request::builder()
        .uri("/auth/logout")
        .body(Body::empty())
        .unwrap();

    let logout_resp = app.oneshot(logout_req).await.unwrap();
    assert_eq!(logout_resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        logout_resp.headers().get(header::LOCATION).unwrap(),
        "/login"
    );
    let set_cookie = logout_resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(set_cookie.contains("__session=") && set_cookie.contains("Max-Age=0"));
}

#[tokio::test]
async fn test_e2e_guest_mode_flow() {
    let mock_upstream = MockServer::start().await;

    // Upstream mocks for entries & unread counters
    Mock::given(method("GET"))
        .and(path("/v1/feeds/counters"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "reads": {},
            "unreads": { "1": 2 }
        })))
        .mount(&mock_upstream)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1/entries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total": 1,
            "entries": [
                {
                    "id": 888,
                    "feed_id": 1,
                    "title": "Guest Flow Entry",
                    "url": "https://example.com/guest",
                    "content": "Full content of the guest test article.",
                    "published_at": "Today",
                    "status": "unread",
                    "starred": false,
                    "reading_time": 1
                }
            ]
        })))
        .mount(&mock_upstream)
        .await;

    // Notice: NO PUT /v1/entries mock is mounted!
    // If guest mode attempted to mutate upstream, wiremock would fail with unmatched request!

    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        app_base_url: "http://127.0.0.1:3000".to_string(),
        auth_enabled: false,
        session_secret: "guest-mode-session-secret-at-least-32-chars-long!".to_string(),
        oidc_issuer_url: "https://auth-disabled.local".to_string(),
        oidc_client_id: "guest".to_string(),
        oidc_client_secret: "guest".to_string(),
        oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
        miniflux_base_url: mock_upstream.uri(),
        miniflux_api_key: "api-key".to_string(),
        ai_enabled: false,
        llm_cli_cmd: "llm".to_string(),
        llm_model: "gpt-4o-mini".to_string(),
        llm_extra_args: vec![],
        database_url: ":memory:".to_string(),
        llm_provider: "openai".to_string(),
        llm_api_key: None,
        llm_api_base_url: None,
    });

    let client = MinifluxClient::new(mock_upstream.uri(), "api-key".to_string());
    let schema = build_schema(client);
    let app = build_app(config, schema).await;

    // 1. Root page serves HTML shell without auth cookies
    let root_req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let root_resp = app.clone().oneshot(root_req).await.unwrap();
    assert_eq!(root_resp.status(), StatusCode::OK);
    let root_body = String::from_utf8(
        root_resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(root_body.contains("my_news_guest_db"));
    assert!(root_body.contains("read_entries"));
    assert!(!root_body.contains("hx-post=\"/auth/logout\""));

    // 2. Auth routes redirect to root
    let login_req = Request::builder()
        .uri("/login")
        .body(Body::empty())
        .unwrap();
    let login_resp = app.clone().oneshot(login_req).await.unwrap();
    assert_eq!(login_resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(login_resp.headers().get(header::LOCATION).unwrap(), "/");

    // 3. Guest HTMX entries request succeeds without login redirect
    let list_req = Request::builder()
        .uri("/hx/entries")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body = String::from_utf8(
        list_resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(list_body.contains("Guest Flow Entry"));

    // 4. Guest mark read returns 403 Forbidden without calling upstream Miniflux
    let read_req = Request::builder()
        .method("POST")
        .uri("/hx/entries/888/read?status=read")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let read_resp = app.clone().oneshot(read_req).await.unwrap();
    assert_eq!(read_resp.status(), StatusCode::FORBIDDEN);

    // 5. DDoS oversized body protection (> 64KB) returns 413 Payload Too Large
    let oversized = format!("entry_ids={}", "a".repeat(70 * 1024));
    let ddos_req = Request::builder()
        .method("POST")
        .uri("/hx/mark-page-read")
        .header("content-type", "application/x-www-form-urlencoded")
        .header("HX-Request", "true")
        .body(Body::from(oversized))
        .unwrap();
    let ddos_resp = app.clone().oneshot(ddos_req).await.unwrap();
    assert_eq!(ddos_resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn test_mutation_endpoints_reject_with_403_when_auth_disabled() {
    let mock_upstream = MockServer::start().await;

    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        app_base_url: "http://127.0.0.1:3000".to_string(),
        auth_enabled: false,
        session_secret: "guest-mode-session-secret-at-least-32-chars-long!".to_string(),
        oidc_issuer_url: "https://auth-disabled.local".to_string(),
        oidc_client_id: "guest".to_string(),
        oidc_client_secret: "guest".to_string(),
        oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
        miniflux_base_url: mock_upstream.uri(),
        miniflux_api_key: "api-key".to_string(),
        ai_enabled: false,
        llm_cli_cmd: "llm".to_string(),
        llm_model: "gpt-4o-mini".to_string(),
        llm_extra_args: vec![],
        database_url: ":memory:".to_string(),
        llm_provider: "openai".to_string(),
        llm_api_key: None,
        llm_api_base_url: None,
    });

    let client = MinifluxClient::new(mock_upstream.uri(), "api-key".to_string());
    let schema = build_schema(client);
    let app = build_app(config, schema).await;

    // 1. Mark entry read returns 403
    let req = Request::builder()
        .method("POST")
        .uri("/hx/entries/123/read?status=read")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 2. Toggle bookmark returns 403
    let req = Request::builder()
        .method("POST")
        .uri("/hx/entries/123/bookmark")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 3. Mark page read returns 403
    let req = Request::builder()
        .method("POST")
        .uri("/hx/mark-page-read")
        .header("content-type", "application/x-www-form-urlencoded")
        .header("HX-Request", "true")
        .body(Body::from("entry_ids=123,124"))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 4. Mark feed read returns 403
    let req = Request::builder()
        .method("POST")
        .uri("/hx/feeds/1/mark-all-read")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 5. Mark all read returns 403
    let req = Request::builder()
        .method("POST")
        .uri("/hx/mark-all-read")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_executive_brief_ineligible_views_and_empty_guard() {
    let mock_upstream = MockServer::start().await;

    let _db_file = tempfile::NamedTempFile::new().unwrap();

    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        app_base_url: "http://127.0.0.1:3000".to_string(),
        auth_enabled: false,
        session_secret: "guest-mode-session-secret-at-least-32-chars-long!".to_string(),
        oidc_issuer_url: "https://auth-disabled.local".to_string(),
        oidc_client_id: "guest".to_string(),
        oidc_client_secret: "guest".to_string(),
        oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
        miniflux_base_url: mock_upstream.uri(),
        miniflux_api_key: "api-key".to_string(),
        ai_enabled: true,
        llm_cli_cmd: "echo".to_string(),
        llm_model: "gpt-4o-mini".to_string(),
        llm_extra_args: vec![],
        database_url: ":memory:".to_string(),
        llm_provider: "openai".to_string(),
        llm_api_key: None,
        llm_api_base_url: None,
    });

    let client = MinifluxClient::new(
        config.miniflux_base_url.clone(),
        config.miniflux_api_key.clone(),
    );
    let schema = build_schema(client);
    let app = build_app(config, schema).await;

    // 1. status=today view is prohibited: returns 200 OK with empty body
    let req = Request::builder()
        .uri("/hx/executive-brief?status=today")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert!(body.is_empty(), "today view must return empty response");

    // 2. Active search query is prohibited: returns 200 OK with empty body
    let req = Request::builder()
        .uri("/hx/executive-brief?status=unread&search=rust")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert!(body.is_empty(), "search query must return empty response");

    // 3. Empty entries guard: returns 200 OK with empty body when upstream has 0 entries
    Mock::given(method("GET"))
        .and(path("/v1/entries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total": 0,
            "entries": []
        })))
        .mount(&mock_upstream)
        .await;

    let req = Request::builder()
        .uri("/hx/executive-brief?status=unread")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert!(body.is_empty(), "empty entries must return empty response");
}

#[tokio::test]
async fn test_executive_brief_cache_hit_and_oob_swaps() {
    let mock_upstream = MockServer::start().await;

    let _db_file = tempfile::NamedTempFile::new().unwrap();

    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        app_base_url: "http://127.0.0.1:3000".to_string(),
        auth_enabled: false,
        session_secret: "guest-mode-session-secret-at-least-32-chars-long!".to_string(),
        oidc_issuer_url: "https://auth-disabled.local".to_string(),
        oidc_client_id: "guest".to_string(),
        oidc_client_secret: "guest".to_string(),
        oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
        miniflux_base_url: mock_upstream.uri(),
        miniflux_api_key: "api-key".to_string(),
        ai_enabled: true,
        llm_cli_cmd: "echo".to_string(),
        llm_model: "gpt-4o-mini".to_string(),
        llm_extra_args: vec![],
        database_url: ":memory:".to_string(),
        llm_provider: "openai".to_string(),
        llm_api_key: None,
        llm_api_base_url: None,
    });

    let client = MinifluxClient::new(
        config.miniflux_base_url.clone(),
        config.miniflux_api_key.clone(),
    );
    let schema = build_schema(client);
    let ai_service = Arc::new(server::ai::AiService::new(&config).await.unwrap());

    // Pre-populate DB with cached page brief and entry summary
    let entry_id = 999_i64;
    let page_key = server::ai::compute_page_key("unread", &[entry_id]);
    ai_service
        .db
        .save_page_brief_by_key(&page_key, "Cached Executive Synthesis")
        .await
        .unwrap();
    ai_service
        .db
        .save_summary_and_title(
            entry_id,
            Some("Punchy AI Headline"),
            "Summary of article 999",
            "gpt-4o-mini",
        )
        .await
        .unwrap();

    let app = server::app::build_app_with_ai(config, schema, ai_service);

    Mock::given(method("GET"))
        .and(path("/v1/entries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total": 1,
            "entries": [{
                "id": entry_id,
                "user_id": 1,
                "feed_id": 10,
                "status": "unread",
                "hash": "hash-999",
                "title": "Original Long Headline",
                "url": "https://example.com/999",
                "comments_url": "",
                "published_at": "2026-11-22T08:00:00Z",
                "created_at": "2026-11-22T08:00:00Z",
                "content": "<p>Content</p>",
                "author": "Author",
                "share_code": "",
                "starred": false,
                "reading_time": 2,
                "feed": {
                    "id": 10,
                    "user_id": 1,
                    "feed_url": "https://example.com/rss",
                    "site_url": "https://example.com",
                    "title": "Tech News",
                    "checked_at": "2026-11-22T08:00:00Z",
                    "category": { "id": 1, "user_id": 1, "title": "Tech" }
                }
            }]
        })))
        .mount(&mock_upstream)
        .await;

    let req = Request::builder()
        .uri("/hx/executive-brief?status=unread")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify Cache-Control header
    let cache_control = resp
        .headers()
        .get(header::CACHE_CONTROL)
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(cache_control, "no-cache, no-store, must-revalidate");

    let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Verify Executive Brief Card
    assert!(body_str.contains("id=\"page-executive-brief\""));
    assert!(body_str.contains("Executive Brief"));
    assert!(body_str.contains("Cached Executive Synthesis"));

    // Verify OOB Title Swap with Annuaire typography
    assert!(body_str.contains("id=\"entry-title-999\""));
    assert!(body_str.contains("hx-swap-oob=\"outerHTML\""));
    assert!(body_str.contains("class=\"article-title text-sm font-medium font-sans leading-snug line-clamp-2 text-foreground group-hover:text-foreground/80 transition-colors\""));
    assert!(body_str.contains(r#"title="Original Long Headline""#));
    assert!(body_str.contains("Punchy AI Headline"));
}

#[tokio::test]
async fn test_e2e_page_load_and_detail_drawer_rewrite_flow() {
    let mock_upstream = MockServer::start().await;

    // Mock Miniflux entry 1
    Mock::given(method("GET"))
        .and(path("/v1/entries/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "user_id": 1,
            "feed_id": 10,
            "status": "unread",
            "hash": "hash-1",
            "title": "Lead Story 1",
            "url": "https://example.com/1",
            "comments_url": "",
            "published_at": "2026-11-24T08:00:00Z",
            "created_at": "2026-11-24T08:00:00Z",
            "content": "<p>Content for lead story 1</p>",
            "author": "Editor",
            "share_code": "",
            "starred": false,
            "reading_time": 2,
            "feed": {
                "id": 10,
                "user_id": 1,
                "feed_url": "https://example.com/rss",
                "site_url": "https://example.com",
                "title": "Tech News",
                "checked_at": "2026-11-24T08:00:00Z",
                "category": { "id": 1, "user_id": 1, "title": "Tech" }
            }
        })))
        .mount(&mock_upstream)
        .await;

    // Mock Miniflux entry 2
    Mock::given(method("GET"))
        .and(path("/v1/entries/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 2,
            "user_id": 1,
            "feed_id": 10,
            "status": "unread",
            "hash": "hash-2",
            "title": "Second Story 2",
            "url": "https://example.com/2",
            "comments_url": "",
            "published_at": "2026-11-24T07:00:00Z",
            "created_at": "2026-11-24T07:00:00Z",
            "content": "<p>Content for second story 2</p>",
            "author": "Reporter",
            "share_code": "",
            "starred": false,
            "reading_time": 3,
            "feed": {
                "id": 10,
                "user_id": 1,
                "feed_url": "https://example.com/rss",
                "site_url": "https://example.com",
                "title": "Tech News",
                "checked_at": "2026-11-24T07:00:00Z",
                "category": { "id": 1, "user_id": 1, "title": "Tech" }
            }
        })))
        .mount(&mock_upstream)
        .await;

    let _db_file = tempfile::NamedTempFile::new().unwrap();

    let mut script_file = tempfile::NamedTempFile::new().unwrap();
    let script_content =
        "#!/bin/sh\nprintf '%b' 'Lazy loaded rewrite for story 2 lead.\\n\\n• Detail point.'\n";
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    script_file.write_all(script_content.as_bytes()).unwrap();
    let mut perms = std::fs::metadata(script_file.path()).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(script_file.path(), perms).unwrap();

    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        app_base_url: "http://127.0.0.1:3000".to_string(),
        auth_enabled: false,
        session_secret: "super-secret-key-that-is-at-least-32-bytes-long!".to_string(),
        oidc_issuer_url: "https://id.example.com".to_string(),
        oidc_client_id: "test-client".to_string(),
        oidc_client_secret: "test-secret".to_string(),
        oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
        miniflux_base_url: mock_upstream.uri(),
        miniflux_api_key: "api-key".to_string(),
        ai_enabled: true,
        llm_cli_cmd: script_file.path().to_str().unwrap().to_string(),
        llm_model: "test-model".to_string(),
        llm_extra_args: vec![],
        database_url: ":memory:".to_string(),
        llm_provider: "openai".to_string(),
        llm_api_key: None,
        llm_api_base_url: None,
    });

    let client = MinifluxClient::new(mock_upstream.uri(), "api-key".to_string());
    let schema = build_schema(client);
    let ai_service = Arc::new(server::ai::AiService::new(&config).await.unwrap());

    // Story 1: Eager rewrite cached
    ai_service
        .db
        .save_summary_and_title_with_rewrite(
            1,
            Some("Punchy Headline 1"),
            "Executive brief for story 1.",
            Some("Eager top-1 rewrite for lead story.\n\n• Impact bullet."),
            "test-model",
        )
        .await
        .unwrap();

    // Story 2: Only brief cached, NO rewrite
    ai_service
        .db
        .save_summary_and_title(
            2,
            Some("Punchy Headline 2"),
            "Executive brief for story 2.",
            "test-model",
        )
        .await
        .unwrap();

    let app = server::app::build_app_with_ai(config, schema, ai_service);

    // 1. Open drawer for Story 1 -> cached rewrite displays directly without skeleton
    let req1 = Request::builder()
        .uri("/hx/entries/1")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);
    let body1 = String::from_utf8(
        resp1
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body1.contains("Eager top-1 rewrite for lead story."));
    assert!(
        !body1.contains("hx-get=\"/hx/entries/1/rewrite\""),
        "Cached rewrite must not load skeleton"
    );

    // 2. Open drawer for Story 2 -> renders executive brief and skeleton loader with hx-get
    let req2 = Request::builder()
        .uri("/hx/entries/2")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let body2 = String::from_utf8(
        resp2
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body2.contains("Executive brief for story 2."));
    assert!(body2.contains("hx-get=\"/hx/entries/2/rewrite\""));
    assert!(body2.contains("animate-pulse"));

    // 3. Trigger HTMX lazy-load route for Story 2 -> generates on-demand rewrite
    let req3 = Request::builder()
        .uri("/hx/entries/2/rewrite")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp3 = app.clone().oneshot(req3).await.unwrap();
    assert_eq!(resp3.status(), StatusCode::OK);
    let body3 = String::from_utf8(
        resp3
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body3.contains("id=\"ai-rewrite-box\""));
    assert!(body3.contains("Lazy loaded rewrite for story 2 lead."));

    // 4. Verify static CSS route
    let req4 = Request::builder()
        .uri("/static/annuaire-theme.css")
        .body(Body::empty())
        .unwrap();
    let resp4 = app.clone().oneshot(req4).await.unwrap();
    assert_eq!(resp4.status(), StatusCode::OK);
    let ct = resp4
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.starts_with("text/css"));
}

#[tokio::test]
async fn test_e2e_detail_drawer_analysis_flow() {
    let mock_upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/entries/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99,
            "user_id": 1,
            "feed_id": 10,
            "title": "Story 99 Title",
            "url": "https://example.com/99",
            "comments_url": "",
            "author": "Tech Author",
            "content": "<p>Raw article body that must never be shown on details page.</p>",
            "hash": "hash99",
            "published_at": "2026-11-24T00:00:00Z",
            "created_at": "2026-11-24T00:00:00Z",
            "status": "unread",
            "starred": false,
            "reading_time": 2,
            "feed": {
                "id": 10,
                "user_id": 1,
                "feed_url": "https://example.com/feed",
                "site_url": "https://example.com",
                "title": "Tech Feed"
            }
        })))
        .mount(&mock_upstream)
        .await;

    let db_path = "mock:e2e_analysis_99";
    let db = server::ai::AiDb::new(db_path).await.unwrap();
    db.save_summary_and_title_with_analysis(
        99,
        Some("Story 99 Title"),
        "Executive brief 99.",
        Some("Detailed inverted pyramid rewrite for 99."),
        Some("Direct disruption to mobile software ecosystems."),
        Some("Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu"),
        "test-model",
    )
    .await
    .unwrap();

    let config = Arc::new(Config {
        host: "127.0.0.1".to_string(),
        port: 3000,
        app_base_url: "http://127.0.0.1:3000".to_string(),
        auth_enabled: false,
        session_secret: "super-secret-key-that-is-at-least-32-bytes-long!".to_string(),
        oidc_issuer_url: "https://id.example.com".to_string(),
        oidc_client_id: "test-client".to_string(),
        oidc_client_secret: "test-secret".to_string(),
        oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
        miniflux_base_url: mock_upstream.uri(),
        miniflux_api_key: "api-key".to_string(),
        ai_enabled: true,
        llm_cli_cmd: "llm".to_string(),
        llm_model: "test-model".to_string(),
        llm_extra_args: vec![],
        database_url: db_path.to_string(),
        llm_provider: "openai".to_string(),
        llm_api_key: None,
        llm_api_base_url: None,
    });

    let client = MinifluxClient::new(mock_upstream.uri(), "api-key".to_string());
    let schema = build_schema(client);
    let app = server::app::build_app(config, schema).await;

    // 1. Open drawer for Entry 99
    let req = Request::builder()
        .uri("/hx/entries/99")
        .header("HX-Request", "true")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = String::from_utf8(
        resp.into_body().collect().await.unwrap().to_bytes().to_vec(),
    )
    .unwrap();

    // Must show Executive Brief, Impact Card, Fallacy Card, and Rewrite
    assert!(body.contains("Executive Brief"));
    assert!(body.contains("Executive brief 99."));
    assert!(body.contains("Impact Analysis"));
    assert!(body.contains("Direct disruption to mobile software ecosystems."));
    assert!(body.contains("Logical Fallacy & Objectivity Check"));
    assert!(body.contains("Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu"));
    assert!(body.contains("Detailed inverted pyramid rewrite for 99."));

    // Must NEVER show raw original content
    assert!(!body.contains("Raw article body that must never be shown on details page."));
}
