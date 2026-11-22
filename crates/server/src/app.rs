use crate::ai::AiService;
use crate::auth::handlers::{callback_handler, login_handler, logout_handler};
use crate::cache::MinifluxCache;
use crate::config::Config;
use crate::rate_limit::{rate_limit_middleware, RateLimiter};
use crate::routes::entries::{
    executive_brief_htmx, get_entry_ai_summary_htmx, get_entry_detail_htmx, list_entries_htmx,
    mark_all_read_htmx, mark_entry_read_htmx, mark_feed_read_htmx, mark_page_read_htmx,
    toggle_bookmark_htmx,
};
use crate::routes::graphql::graphql_handler;
use crate::routes::pages::{index_page, login_page};
use axum::extract::{DefaultBodyLimit, FromRef};
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::Router;
use miniflux_client::schema::AppSchema;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub schema: AppSchema,
    pub ai_service: Arc<AiService>,
    pub cache: Arc<MinifluxCache>,
    pub rate_limiter: Arc<RateLimiter>,
}

impl FromRef<AppState> for Arc<Config> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for AppSchema {
    fn from_ref(state: &AppState) -> Self {
        state.schema.clone()
    }
}

impl FromRef<AppState> for Arc<AiService> {
    fn from_ref(state: &AppState) -> Self {
        state.ai_service.clone()
    }
}

impl FromRef<AppState> for Arc<MinifluxCache> {
    fn from_ref(state: &AppState) -> Self {
        state.cache.clone()
    }
}

impl FromRef<AppState> for Arc<RateLimiter> {
    fn from_ref(state: &AppState) -> Self {
        state.rate_limiter.clone()
    }
}

pub fn build_app(config: Arc<Config>, schema: AppSchema) -> Router {
    let ai_service = Arc::new(AiService::new(&config).unwrap_or_else(|_| {
        let fallback_path = std::env::temp_dir()
            .join(format!("ai_summaries_{}.db", std::process::id()))
            .to_string_lossy()
            .to_string();
        AiService::new(&Config {
            ai_db_path: fallback_path,
            ..(*config).clone()
        })
        .expect("Failed to initialize fallback AiService")
    }));
    build_app_with_ai(config, schema, ai_service)
}

pub fn build_app_with_ai(
    config: Arc<Config>,
    schema: AppSchema,
    ai_service: Arc<AiService>,
) -> Router {
    let cache = Arc::new(MinifluxCache::default());
    let rate_limiter = Arc::new(RateLimiter::default());
    let state = AppState {
        config,
        schema,
        ai_service,
        cache,
        rate_limiter: rate_limiter.clone(),
    };

    Router::new()
        .route("/", get(index_page))
        .route("/entries/{id}", get(index_page))
        .route("/health", get(|| async { axum::Json(serde_json::json!({"status": "ok"})) }))
        .route("/login", get(login_page))
        .route("/auth/login", get(login_handler))
        .route("/auth/callback", get(callback_handler))
        .route("/auth/logout", get(logout_handler))
        .route("/hx/entries", get(list_entries_htmx))
        .route("/hx/entries/{id}", get(get_entry_detail_htmx))
        .route("/hx/entries/{id}/bookmark", post(toggle_bookmark_htmx))
        .route("/hx/entries/{id}/read", post(mark_entry_read_htmx))
        .route("/hx/entries/{id}/ai-summary", get(get_entry_ai_summary_htmx))
        .route("/hx/executive-brief", get(executive_brief_htmx))
        .route("/hx/feeds/{id}/mark-all-read", post(mark_feed_read_htmx))
        .route("/hx/mark-all-read", post(mark_all_read_htmx))
        .route("/hx/mark-page-read", post(mark_page_read_htmx))
        .route("/graphql", post(graphql_handler))
        .layer(from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(DefaultBodyLimit::max(64 * 1024))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use miniflux_client::client::MinifluxClient;
    use miniflux_client::schema::build_schema;
    use serde_json::Value;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
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
            miniflux_base_url: "http://127.0.0.1:8080".to_string(),
            miniflux_api_key: "api-key".to_string(),
            ai_enabled: false,
            llm_cli_cmd: "llm".to_string(),
            llm_model: "gpt-4o-mini".to_string(),
            llm_extra_args: vec![],
            ai_db_path: ":memory:".to_string(),
        });

        let client = MinifluxClient::new("http://127.0.0.1:8080".to_string(), "api-key".to_string());
        let schema = build_schema(client);
        let app = build_app(config, schema);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }
}

