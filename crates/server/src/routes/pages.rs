use crate::ai::AiService;
use crate::auth::AuthenticatedUser;
use crate::cache::MinifluxCache;
use crate::config::Config;
use crate::error::AppError;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use miniflux_client::schema::AppSchema;
use std::sync::Arc;
use std::time::Duration;
use ui_components::templates::{IndexTemplate, LoginTemplate};
use askama::Template;

pub async fn login_page(State(config): State<Arc<Config>>) -> Response {
    if !config.auth_enabled {
        return Redirect::to("/").into_response();
    }
    Html(LoginTemplate.render().unwrap_or_default()).into_response()
}

pub async fn index_page(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    State(cache): State<Arc<MinifluxCache>>,
    State(ai_service): State<Arc<AiService>>,
) -> Result<Html<String>, AppError> {
    let cache_key = if config.auth_enabled {
        "index_page_entries_auth"
    } else {
        "index_page_entries_guest"
    };
    let data = if let Some(cached) = cache.get(cache_key).await {
        cached
    } else {
        let d = if config.auth_enabled {
            let gql = r#"query {
                unreadCounter
                todayCounter
                entriesResult(filter: { limit: 50, status: UNREAD }) {
                    total
                    entries {
                        id
                        title
                        summary
                        publishedAt
                        readingTime
                        isBookmarked
                        url
                        status
                    }
                }
            }"#;
            let res = schema.execute(gql).await;
            res.data.into_json().unwrap_or_default()
        } else {
            let start_of_today = chrono::Utc::now()
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp();
            let gql = format!(r#"query {{
                unreadCounter
                todayCounter
                entriesResult(filter: {{ limit: 50, publishedAfter: {start_of_today} }}) {{
                    total
                    entries {{
                        id
                        title
                        summary
                        publishedAt
                        readingTime
                        isBookmarked
                        url
                        status
                    }}
                }}
            }}"#);
            let res = schema.execute(&gql).await;
            res.data.into_json().unwrap_or_default()
        };
        cache.insert_with_ttl(cache_key.to_string(), d.clone(), Duration::from_secs(30)).await;
        d
    };

    let mut unread_count = if config.auth_enabled {
        data.get("unreadCounter")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32
    } else {
        data.get("todayCounter")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32
    };

    let today_count = data
        .get("todayCounter")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let entries_result = data.get("entriesResult");
    let total = entries_result
        .and_then(|r| r.get("total"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    if unread_count == 0 && total > 0 {
        unread_count = total as i32;
    }

    let limit: i64 = 50;
    let offset: i64 = 0;
    let total_pages = if total <= 0 {
        1
    } else {
        (total + limit - 1) / limit
    };
    let current_page = (offset / limit) + 1;
    let has_prev = offset > 0;
    let has_next = offset + limit < total;
    let prev_offset = (offset - limit).max(0);
    let next_offset = offset + limit;

    let mut view_models: Vec<ui_components::templates::EntryViewModel> = entries_result
        .and_then(|r| r.get("entries"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(ui_components::templates::EntryViewModel {
                        id: item.get("id")?.as_str()?.parse().ok()?,
                        title: item.get("title")?.as_str()?.to_string(),
                        suggested_title: None,
                        feed_title: "Feed".to_string(),
                        published_at: item.get("publishedAt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        reading_time: item.get("readingTime").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                        summary: item.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        url: item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        author: None,
                        is_read: if config.auth_enabled {
                            item.get("status").and_then(|v| v.as_str()) == Some("READ")
                        } else {
                            false
                        },
                        is_bookmarked: item.get("isBookmarked").and_then(|v| v.as_bool()).unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let (ai_summaries, _top_executive_brief) = if config.ai_enabled {
        let ids: Vec<i64> = view_models.iter().map(|e| e.id).collect();
        let map = ai_service.db.get_batch_summaries(&ids).await;
        let brief = ai_service.db.get_latest_page_brief().await;
        (map, brief)
    } else {
        (std::collections::HashMap::new(), None)
    };

    for entry in &mut view_models {
        if let Some(record) = ai_summaries.get(&entry.id) {
            entry.suggested_title = record.suggested_title.clone();
        }
    }

    if config.ai_enabled {
        let candidates: Vec<crate::ai::prefetch::PrefetchCandidate> = view_models
            .iter()
            .filter(|e| !e.is_read)
            .map(|e| crate::ai::prefetch::PrefetchCandidate {
                id: e.id,
                title: e.title.clone(),
                url: e.url.clone(),
                teaser: e.summary.clone(),
                feed_title: e.feed_title.clone(),
                published_at: e.published_at.clone(),
            })
            .collect();
        crate::ai::prefetch::AiPrefetchService::new(ai_service.clone())
            .spawn_prefetch_job(candidates);
    }

    let list_template = ui_components::templates::EntryListTemplate {
        entries: view_models,
        top_executive_brief: None,
        unread_count: None,
        current_page,
        total_pages,
        has_prev,
        has_next,
        prev_offset,
        next_offset,
        status: "unread".to_string(),
        search: "".to_string(),
        limit,
        is_brief_eligible: config.ai_enabled,
        offset: 0,
        feed_id: None,
        category_id: None,
    };
    let entries_list_html = list_template.render().unwrap_or_default();

    let template = IndexTemplate {
        unread_count,
        today_count,
        entries_list_html,
        ai_enabled: config.ai_enabled,
        auth_enabled: config.auth_enabled,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Html(html))
}
