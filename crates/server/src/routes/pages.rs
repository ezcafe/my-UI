use crate::auth::AuthenticatedUser;
use crate::config::Config;
use crate::error::AppError;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use miniflux_client::schema::AppSchema;
use std::sync::Arc;
use ui_components::templates::{IndexTemplate, LoginTemplate};
use askama::Template;

pub async fn login_page() -> impl IntoResponse {
    let html = LoginTemplate.render().unwrap_or_default();
    Html(html)
}

pub async fn index_page(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
) -> Result<Html<String>, AppError> {
    let unread_res = schema.execute("{ unreadCounter }").await;
    let unread_count = unread_res.data.into_json()
        .ok()
        .and_then(|d| d.get("unreadCounter").and_then(|v| v.as_i64()))
        .unwrap_or(0) as i32;

    let entries_res = schema.execute("{ entries(filter: { limit: 50, status: UNREAD }) { id title summary publishedAt readingTime isBookmarked url status } }").await;
    let entries_json = entries_res.data.into_json().unwrap_or_default();

    let view_models: Vec<ui_components::templates::EntryViewModel> = entries_json
        .get("entries")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(ui_components::templates::EntryViewModel {
                        id: item.get("id")?.as_str()?.parse().ok()?,
                        title: item.get("title")?.as_str()?.to_string(),
                        feed_title: "Feed".to_string(),
                        published_at: item.get("publishedAt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        reading_time: item.get("readingTime").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                        summary: item.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        url: item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        author: None,
                        is_read: item.get("status").and_then(|v| v.as_str()) == Some("READ"),
                        is_bookmarked: item.get("isBookmarked").and_then(|v| v.as_bool()).unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let list_template = ui_components::templates::EntryListTemplate {
        entries: view_models,
        unread_count: None,
    };
    let entries_list_html = list_template.render().unwrap_or_default();

    let template = IndexTemplate {
        unread_count,
        entries_list_html,
        ai_enabled: config.ai_enabled,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Html(html))
}
