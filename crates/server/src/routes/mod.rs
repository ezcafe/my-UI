pub mod entries;
pub mod graphql;
pub mod pages;

#[cfg(test)]
mod tests {
    use crate::app::build_app;
    use crate::auth::session::{create_session_cookie_value, SessionData};
    use crate::config::Config;
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use http_body_util::BodyExt;
    use miniflux_client::client::MinifluxClient;
    use miniflux_client::schema::build_schema;
    use serde_json::json;
    use std::sync::Arc;
    use tower::ServiceExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_test_app(server_uri: &str) -> (axum::Router, String) {
        let secret = "super-secret-key-that-is-at-least-32-bytes-long!";
        let config = Arc::new(Config {
            host: "127.0.0.1".to_string(),
            port: 3000,
            app_base_url: "http://127.0.0.1:3000".to_string(),
            auth_enabled: true,
            session_secret: secret.to_string(),
            oidc_issuer_url: "https://id.example.com".to_string(),
            oidc_client_id: "test-client".to_string(),
            oidc_client_secret: "test-secret".to_string(),
            oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
            miniflux_base_url: server_uri.to_string(),
            miniflux_api_key: "api-key".to_string(),
            ai_enabled: false,
            llm_cli_cmd: "llm".to_string(),
            llm_model: "gpt-4o-mini".to_string(),
            llm_extra_args: vec![],
            ai_db_path: "data/summaries.db".to_string(),
        });

        let client = MinifluxClient::new(server_uri.to_string(), "api-key".to_string());
        let schema = build_schema(client);
        let app = build_app(config, schema);

        let session = SessionData {
            user_id: "u_1".to_string(),
            username: "tester".to_string(),
            access_token: "tok_123".to_string(),
            refresh_token: None,
            expires_at: 1800000000,
        };
        let enc = create_session_cookie_value(&session, secret).unwrap();
        let cookie_header = format!("__session={}", enc);

        (app, cookie_header)
    }

    #[tokio::test]
    async fn test_get_root_page_serves_html_shell() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "reads": {},
                "unreads": { "1": 3 }
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 0,
                "entries": []
            })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req = Request::builder()
            .uri("/")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("My News"));
        assert!(html.contains("id=\"unread-pill\""));
    }

    #[tokio::test]
    async fn test_hx_entries_filter_query_params() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 1,
                "entries": [
                    {
                        "id": 88,
                        "feed_id": 1,
                        "title": "Filtered Article",
                        "url": "https://example.com/filtered",
                        "content": "A filtered post content.",
                        "published_at": "Today",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 2
                    }
                ]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "reads": {}, "unreads": {} })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req = Request::builder()
            .uri("/hx/entries?status=unread&limit=25&offset=50")
            .header("HX-Request", "true")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("Filtered Article"));
    }

    #[tokio::test]
    async fn test_hx_entries_filter_today_and_search() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 1,
                "entries": [
                    {
                        "id": 99,
                        "feed_id": 1,
                        "title": "Today's Article",
                        "url": "https://example.com/today",
                        "content": "Today's article content.",
                        "published_at": "Today",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 1
                    }
                ]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "reads": {}, "unreads": {} })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req = Request::builder()
            .uri("/hx/entries?status=today&search=Today")
            .header("HX-Request", "true")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let status = resp.status();
        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert_eq!(status, StatusCode::OK, "Body: {}", html);
        assert!(html.contains("Today&#x27;s Article") || html.contains("Today's Article"), "Actual html: {}", html);
    }

    #[tokio::test]
    async fn test_hx_read_returns_oob_unread_pill() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "reads": {},
                "unreads": { "1": 4 }
            })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req = Request::builder()
            .method("POST")
            .uri("/hx/entries/88/read?status=read")
            .header("HX-Request", "true")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("id=\"entry-88\""));
        assert!(html.contains("id=\"unread-pill\""));
        assert!(html.contains("hx-swap-oob=\"true\""));
        assert!(html.contains("4 unread"));
    }

    #[tokio::test]
    async fn test_hx_bookmark_returns_updated_icon() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/v1/entries/88/bookmark"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req = Request::builder()
            .method("POST")
            .uri("/hx/entries/88/bookmark")
            .header("HX-Request", "true")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("hx-post=\"/hx/entries/88/bookmark\""));
        assert!(html.contains("text-amber-300") || html.contains("fill-amber-400"));
    }

    #[tokio::test]
    async fn test_htmx_error_returns_oob_toast_without_reswapping_target() {
        let server = MockServer::start().await;

        // Upstream returns 500 error
        Mock::given(method("PUT"))
            .and(path("/v1/entries/88/bookmark"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal DB Error"))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req = Request::builder()
            .method("POST")
            .uri("/hx/entries/88/bookmark")
            .header("HX-Request", "true")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get("HX-Reswap").unwrap(), "none");

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("id=\"toast-container\""));
        assert!(html.contains("hx-swap-oob=\"beforeend\""));
    }

    #[tokio::test]
    async fn test_get_entry_detail_htmx_fetches_single_entry() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 123,
                "user_id": 1,
                "feed_id": 10,
                "title": "Specific Deep Article 123",
                "url": "https://example.com/123",
                "content": "<p>Full content of 123 with a <a href=\"https://sub.example.com\">link</a></p>",
                "published_at": "2026-11-19T12:00:00Z",
                "status": "read",
                "starred": true,
                "reading_time": 4,
                "feed": {
                    "id": 10,
                    "title": "Rust Weekly",
                    "site_url": "https://rustweekly.example.com",
                    "feed_url": "https://rustweekly.example.com/rss",
                    "checked_at": "2026-11-19T10:00:00Z",
                    "unread_count": 0
                }
            })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req = Request::builder()
            .method("GET")
            .uri("/hx/entries/123")
            .header("HX-Request", "true")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("Specific Deep Article 123"));
        assert!(html.contains("target=\"_blank\""));
    }

    #[tokio::test]
    async fn test_get_entry_detail_auto_marks_as_read_and_returns_oob_swaps() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries/124"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 124,
                "user_id": 1,
                "feed_id": 10,
                "title": "Specific Unread Article 124",
                "url": "https://example.com/124",
                "content": "<p>Content 124</p>",
                "published_at": "2026-11-19T12:00:00Z",
                "status": "unread",
                "starred": false,
                "reading_time": 2,
                "feed": {
                    "id": 10,
                    "title": "Rust Weekly",
                    "site_url": "https://rustweekly.example.com",
                    "feed_url": "https://rustweekly.example.com/rss",
                    "checked_at": "2026-11-19T10:00:00Z",
                    "unread_count": 1
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "reads": {},
                "unreads": { "10": 9 }
            })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req = Request::builder()
            .method("GET")
            .uri("/hx/entries/124")
            .header("HX-Request", "true")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert!(html.contains("Specific Unread Article 124"));
        assert!(html.contains("id=\"unread-pill\""));
        assert!(html.contains("9 unread"));
        assert!(html.contains("id=\"filter-unread-count\""));
        assert!(html.contains("(9)"));
        assert!(html.contains("id=\"entry-124-dot\""));
        assert!(html.contains("hx-swap-oob=\"true\""));
    }

    #[tokio::test]
    async fn test_mark_feed_read_htmx() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/v1/feeds/9/mark-all-as-read"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "reads": {},
                "unreads": { "9": 0 }
            })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req = Request::builder()
            .method("POST")
            .uri("/hx/feeds/9/mark-all-read")
            .header("HX-Request", "true")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("id=\"unread-pill\""));
        assert!(html.contains("0 unread"));
    }

    #[tokio::test]
    async fn test_hx_bookmark_toggle_to_unbookmarked() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/v1/entries/88/bookmark"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/entries/88"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 88,
                "user_id": 1,
                "feed_id": 1,
                "title": "Article 88",
                "url": "https://example.com/88",
                "content": "<p>Content</p>",
                "published_at": "2026-11-19T10:00:00Z",
                "status": "read",
                "starred": false,
                "reading_time": 1
            })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req = Request::builder()
            .method("POST")
            .uri("/hx/entries/88/bookmark")
            .header("HX-Request", "true")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("hx-post=\"/hx/entries/88/bookmark\""));
        // Since starred is false, fill-amber-400 should not be present
        assert!(!html.contains("fill-amber-400"));
    }

    #[tokio::test]
    async fn test_hx_mark_page_read_endpoint() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 0,
                "entries": []
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "reads": {},
                "unreads": { "1": 0 }
            })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let form_body = "entry_ids=101,102&status=all";
        let req = Request::builder()
            .method("POST")
            .uri("/hx/mark-page-read")
            .header("HX-Request", "true")
            .header(header::COOKIE, cookie)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("unread-pill"));
    }

    async fn setup_test_app_ai(server_uri: &str, ai_enabled: bool) -> (axum::Router, String) {
        let secret = "super-secret-key-that-is-at-least-32-bytes-long!";
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let config = Arc::new(Config {
            host: "127.0.0.1".to_string(),
            port: 3000,
            app_base_url: "http://127.0.0.1:3000".to_string(),
            auth_enabled: true,
            session_secret: secret.to_string(),
            oidc_issuer_url: "https://id.example.com".to_string(),
            oidc_client_id: "test-client".to_string(),
            oidc_client_secret: "test-secret".to_string(),
            oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
            miniflux_base_url: server_uri.to_string(),
            miniflux_api_key: "api-key".to_string(),
            ai_enabled,
            llm_cli_cmd: "non-existent-llm".to_string(),
            llm_model: "gpt-4o-mini".to_string(),
            llm_extra_args: vec![],
            ai_db_path: tmp.path().to_str().unwrap().to_string(),
        });

        let client = MinifluxClient::new(server_uri.to_string(), "api-key".to_string());
        let schema = build_schema(client);
        let app = build_app(config, schema);

        let session = SessionData {
            user_id: "u_1".to_string(),
            username: "tester".to_string(),
            access_token: "tok_123".to_string(),
            refresh_token: None,
            expires_at: 1800000000,
        };
        let enc = create_session_cookie_value(&session, secret).unwrap();
        let cookie_header = format!("__session={}", enc);

        (app, cookie_header)
    }

    #[tokio::test]
    async fn test_hx_entries_ai_status_forwards_search_query() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .and(wiremock::matchers::query_param("search", "Neural"))
            .and(wiremock::matchers::query_param("limit", "15"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 1,
                "entries": [
                    {
                        "id": 501,
                        "feed_id": 10,
                        "title": "Neural Rendering",
                        "url": "https://example.com/neural",
                        "content": "<p>Neural rendering is fast.</p>",
                        "published_at": "2026-11-20T00:00:00Z",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 2,
                        "feed": {
                            "id": 10,
                            "title": "Graphics News",
                            "site_url": "https://example.com",
                            "feed_url": "https://example.com/feed",
                            "checked_at": "2026-11-20T00:00:00Z",
                            "category": { "id": 1, "title": "Tech" }
                        }
                    }
                ]
            })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app_ai(&server.uri(), true).await;

        let req = Request::builder()
            .uri("/hx/entries?status=ai&search=Neural")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert!(html.contains("Graphics News"));
        assert!(html.contains("Neural Rendering"));
        assert!(html.contains("hx-get=\"/hx/entries/501/ai-summary\""));
        assert!(html.contains("hx-trigger=\"revealed\""));
    }

    #[tokio::test]
    async fn test_hx_entries_ai_summary_requires_auth_and_verifies_ownership() {
        let server = MockServer::start().await;
        let (app, cookie) = setup_test_app_ai(&server.uri(), true).await;

        // 1. Unauthenticated request must redirect or fail auth
        let unauth_req = Request::builder()
            .uri("/hx/entries/999/ai-summary")
            .body(Body::empty())
            .unwrap();
        let unauth_resp = app.clone().oneshot(unauth_req).await.unwrap();
        assert!(
            unauth_resp.status() == StatusCode::SEE_OTHER || unauth_resp.status() == StatusCode::UNAUTHORIZED,
            "Unauthenticated request must be redirected or rejected"
        );

        // 2. Upstream returns 404 for entry 999 (ownership or non-existence)
        Mock::given(method("GET"))
            .and(path("/v1/entries/999"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let req = Request::builder()
            .uri("/hx/entries/999/ai-summary")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();

        // Must guarantee 200 outerHTML swap
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("Summary unavailable"));
    }

    #[tokio::test]
    async fn test_hx_entries_ai_summary_guarantees_200_swap_on_failure() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries/777"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 777,
                "feed_id": 2,
                "title": "Quantum Leap in AI",
                "url": "https://invalid-host-unreachable-domain-xyz.com/news",
                "content": "<p>Quantum leap achieved today. Details are emerging rapidly. Extra info follows.</p>",
                "published_at": "2026-11-20T00:00:00Z",
                "status": "unread",
                "starred": false,
                "reading_time": 1
            })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app_ai(&server.uri(), true).await;

        let req = Request::builder()
            .uri("/hx/entries/777/ai-summary")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(html.contains("ai-summary-card"));
        assert!(html.contains("Quantum leap achieved today."));
    }

    #[tokio::test]
    async fn test_list_entries_serves_from_cache_when_cached() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 1,
                "entries": [
                    {
                        "id": 42,
                        "feed_id": 1,
                        "title": "Cached Entry",
                        "url": "https://example.com/cached",
                        "content": "Content",
                        "published_at": "2026-11-20T12:00:00Z",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 1
                    }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "reads": {}, "unreads": {} })))
            .mount(&server)
            .await;

        let (app, cookie) = setup_test_app(&server.uri()).await;

        let req1 = Request::builder()
            .uri("/hx/entries?status=unread")
            .header("HX-Request", "true")
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .unwrap();
        let resp1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(resp1.status(), StatusCode::OK);

        let req2 = Request::builder()
            .uri("/hx/entries?status=unread")
            .header("HX-Request", "true")
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .unwrap();
        let resp2 = app.oneshot(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
    }

    async fn setup_test_app_guest(server_uri: &str) -> axum::Router {
        let secret = "super-secret-key-that-is-at-least-32-bytes-long!";
        let config = Arc::new(Config {
            host: "127.0.0.1".to_string(),
            port: 3000,
            app_base_url: "http://127.0.0.1:3000".to_string(),
            auth_enabled: false,
            session_secret: secret.to_string(),
            oidc_issuer_url: "https://id.example.com".to_string(),
            oidc_client_id: "test-client".to_string(),
            oidc_client_secret: "test-secret".to_string(),
            oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
            miniflux_base_url: server_uri.to_string(),
            miniflux_api_key: "api-key".to_string(),
            ai_enabled: false,
            llm_cli_cmd: "llm".to_string(),
            llm_model: "gpt-4o-mini".to_string(),
            llm_extra_args: vec![],
            ai_db_path: "data/summaries.db".to_string(),
        });

        let client = MinifluxClient::new(server_uri.to_string(), "api-key".to_string());
        let schema = build_schema(client);
        build_app(config, schema)
    }

    #[tokio::test]
    async fn test_get_entry_detail_skips_miniflux_mutation_when_auth_disabled() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 123,
                "feed_id": 1,
                "title": "Guest Detail Article",
                "url": "https://example.com/detail",
                "content": "<p>Content</p>",
                "published_at": "2026-11-20T12:00:00Z",
                "status": "unread",
                "reading_time": 2,
                "starred": false
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "reads": {}, "unreads": { "1": 1 } })))
            .mount(&server)
            .await;

        // Mutation must NOT be called in guest mode
        Mock::given(method("PUT"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;

        let app = setup_test_app_guest(&server.uri()).await;

        let req = Request::builder()
            .uri("/hx/entries/123")
            .header("HX-Request", "true")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_toggle_bookmark_htmx_skips_miniflux_when_auth_disabled() {
        let server = MockServer::start().await;

        // Mutation must NOT be called in guest mode
        Mock::given(method("PUT"))
            .and(path("/v1/entries/88/bookmark"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;

        let app = setup_test_app_guest(&server.uri()).await;

        let req = Request::builder()
            .method("POST")
            .uri("/hx/entries/88/bookmark")
            .header("HX-Request", "true")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mark_entry_read_htmx_skips_miniflux_when_auth_disabled() {
        let server = MockServer::start().await;

        // Mutation must NOT be called in guest mode
        Mock::given(method("PUT"))
            .and(path("/v1/entries/88"))
            .respond_with(ResponseTemplate::new(204))
            .expect(0)
            .mount(&server)
            .await;

        let app = setup_test_app_guest(&server.uri()).await;

        let req = Request::builder()
            .method("POST")
            .uri("/hx/entries/88/read")
            .header("HX-Request", "true")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
