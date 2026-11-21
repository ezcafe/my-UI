use crate::ai::AiService;
use crate::auth::handlers::{callback_handler, login_handler, logout_handler};
use crate::config::Config;
use crate::routes::entries::{
    get_entry_ai_summary_htmx, get_entry_detail_htmx, list_entries_htmx, mark_all_read_htmx,
    mark_entry_read_htmx, mark_feed_read_htmx, mark_page_read_htmx, toggle_bookmark_htmx,
};
use crate::routes::graphql::graphql_handler;
use crate::routes::pages::{index_page, login_page};
use axum::extract::FromRef;
use axum::routing::{get, post};
use axum::Router;
use miniflux_client::schema::AppSchema;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub schema: AppSchema,
    pub ai_service: Arc<AiService>,
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
    let state = AppState {
        config,
        schema,
        ai_service,
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
        .route("/hx/feeds/{id}/mark-all-read", post(mark_feed_read_htmx))
        .route("/hx/mark-all-read", post(mark_all_read_htmx))
        .route("/hx/mark-page-read", post(mark_page_read_htmx))
        .route("/graphql", post(graphql_handler))
        .with_state(state)
}
