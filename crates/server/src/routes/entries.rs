use crate::ai::AiService;
use crate::auth::AuthenticatedUser;
use crate::cache::MinifluxCache;
use crate::config::Config;
use crate::error::{handle_htmx_error, AppError};
use askama::Template;
use axum::extract::{Form, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use miniflux_client::schema::AppSchema;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use ui_components::sanitizer::sanitize_html;
use ui_components::templates::{
    AiGroupedListTemplate, AiSummaryCardTemplate, EntryDetailTemplate, EntryItemTemplate,
    EntryListTemplate, EntryViewModel, ErrorCardTemplate, FeedSummaryGroupViewModel,
};

#[derive(Debug, Deserialize, Default)]
pub struct EntriesFilterQuery {
    pub status: Option<String>,
    pub feed_id: Option<i64>,
    pub category_id: Option<i64>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct MarkPageReadForm {
    pub entry_ids: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub feed_id: Option<i64>,
    pub category_id: Option<i64>,
}

pub fn build_graphql_filter(query: &EntriesFilterQuery, limit_val: i64, offset_val: i64, auth_enabled: bool) -> String {
    let mut filter_parts = Vec::new();

    // Status & Filter mapping
    match query.status.as_deref() {
        Some("all") => {},
        Some("read") => {
            if auth_enabled {
                filter_parts.push("status: READ".to_string());
            }
        },
        Some("starred") => filter_parts.push("starred: true".to_string()),
        Some("today") => {
            let start_of_today = chrono::Utc::now()
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp();
            // User decision 1B: show all published today (both read and unread)
            filter_parts.push(format!("publishedAfter: {}", start_of_today));
        },
        Some("unread") => {
            if auth_enabled {
                filter_parts.push("status: UNREAD".to_string());
            } else {
                let start_of_today = chrono::Utc::now()
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .timestamp();
                filter_parts.push(format!("publishedAfter: {}", start_of_today));
            }
        },
        None if query.search.as_ref().is_some_and(|s| !s.trim().is_empty()) => {
            // Default to all entries when search is provided without explicit status
        },
        _ => {
            if auth_enabled {
                filter_parts.push("status: UNREAD".to_string());
            } else {
                let start_of_today = chrono::Utc::now()
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .timestamp();
                filter_parts.push(format!("publishedAfter: {}", start_of_today));
            }
        }
    }

    if let Some(feed_id) = query.feed_id {
        filter_parts.push(format!(r#"feedId: "{}""#, feed_id));
    }
    if let Some(category_id) = query.category_id {
        filter_parts.push(format!(r#"categoryId: "{}""#, category_id));
    }

    // Search mapping
    if let Some(ref search) = query.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            if let Ok(escaped) = serde_json::to_string(trimmed) {
                filter_parts.push(format!("search: {}", escaped));
            }
        }
    }

    filter_parts.push(format!("limit: {}", limit_val));
    filter_parts.push(format!("offset: {}", offset_val));
    filter_parts.join(", ")
}

pub async fn render_entries_list(
    schema: &AppSchema,
    cache: Option<&MinifluxCache>,
    headers: &HeaderMap,
    query: &EntriesFilterQuery,
    auth_enabled: bool,
) -> Response {
    let limit_val = query.limit.unwrap_or(50);
    let offset_val = query.offset.unwrap_or(0);
    let filter_str = build_graphql_filter(query, limit_val, offset_val, auth_enabled);
    let cache_key = format!("entries:{}:auth:{}", filter_str, auth_enabled);
    let counter_field = if auth_enabled {
        "unreadCounter"
    } else {
        "todayCounter"
    };

    let data = if let Some(cache) = cache {
        if let Some(cached_val) = cache.get(&cache_key).await {
            cached_val
        } else {
            let gql = format!(
                r#"query {{
                    entriesResult(filter: {{ {filter_str} }}) {{
                        total
                        entries {{
                            id
                            title
                            summary
                            publishedAt
                            readingTime
                            status
                            isBookmarked
                            url
                        }}
                    }}
                    {counter_field}
                }}"#
            );

            let res = schema.execute(&gql).await;
            if !res.errors.is_empty() {
                return handle_htmx_error(headers, AppError::Upstream(format!("{:?}", res.errors)));
            }

            let d = match res.data.into_json() {
                Ok(d) => d,
                Err(e) => return handle_htmx_error(headers, AppError::Internal(e.to_string())),
            };
            cache.insert_with_ttl(cache_key, d.clone(), Duration::from_secs(30)).await;
            d
        }
    } else {
        let gql = format!(
            r#"query {{
                entriesResult(filter: {{ {filter_str} }}) {{
                    total
                    entries {{
                        id
                        title
                        summary
                        publishedAt
                        readingTime
                        status
                        isBookmarked
                        url
                    }}
                }}
                {counter_field}
            }}"#
        );

        let res = schema.execute(&gql).await;
        if !res.errors.is_empty() {
            return handle_htmx_error(headers, AppError::Upstream(format!("{:?}", res.errors)));
        }

        match res.data.into_json() {
            Ok(d) => d,
            Err(e) => return handle_htmx_error(headers, AppError::Internal(e.to_string())),
        }
    };

    let unread_count = if auth_enabled {
        data.get("unreadCounter").and_then(|v| v.as_i64()).map(|c| c as i32)
    } else {
        data.get("todayCounter").and_then(|v| v.as_i64()).map(|c| c as i32)
    };

    let entries_result = data.get("entriesResult");
    let total = entries_result
        .and_then(|r| r.get("total"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let limit = limit_val;
    let offset = offset_val;
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

    let entries: Vec<EntryViewModel> = entries_result
        .and_then(|r| r.get("entries"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(EntryViewModel {
                        id: item.get("id")?.as_str()?.parse().ok()?,
                        title: item.get("title")?.as_str()?.to_string(),
                        feed_title: "Feed".to_string(),
                        published_at: item.get("publishedAt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        reading_time: item.get("readingTime").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                        summary: item.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        url: item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        author: None,
                        is_read: if auth_enabled {
                            item.get("status").and_then(|v| v.as_str()) == Some("READ") || query.status.as_deref() == Some("read")
                        } else {
                            false
                        },
                        is_bookmarked: item.get("isBookmarked").and_then(|v| v.as_bool()).unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let template = EntryListTemplate {
        entries,
        unread_count,
        current_page,
        total_pages,
        has_prev,
        has_next,
        prev_offset,
        next_offset,
        status: query.status.clone().unwrap_or_else(|| "unread".to_string()),
        search: query.search.clone().unwrap_or_default(),
        limit,
    };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => handle_htmx_error(headers, AppError::Internal(e.to_string())),
    }
}

pub async fn render_ai_entries_list(
    schema: &AppSchema,
    headers: &HeaderMap,
    query: &EntriesFilterQuery,
) -> Response {
    let mut filter_parts = vec!["limit: 15".to_string()];
    if let Some(ref search) = query.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            if let Ok(escaped) = serde_json::to_string(trimmed) {
                filter_parts.push(format!("search: {}", escaped));
            }
        }
    }
    if let Some(feed_id) = query.feed_id {
        filter_parts.push(format!(r#"feedId: "{}""#, feed_id));
    }
    if let Some(category_id) = query.category_id {
        filter_parts.push(format!(r#"categoryId: "{}""#, category_id));
    }

    let filter_str = filter_parts.join(", ");
    let gql = format!(
        r#"query {{
            entries(filter: {{ {filter_str} }}) {{
                id
                title
                summary
                publishedAt
                readingTime
                status
                isBookmarked
                url
                feed {{
                    title
                }}
            }}
        }}"#
    );

    let res = schema.execute(&gql).await;
    if !res.errors.is_empty() {
        return handle_htmx_error(headers, AppError::Upstream(format!("{:?}", res.errors)));
    }

    let data = match res.data.into_json() {
        Ok(d) => d,
        Err(e) => return handle_htmx_error(headers, AppError::Internal(e.to_string())),
    };

    let entries: Vec<EntryViewModel> = data
        .get("entries")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let feed_title = item.get("feed")
                        .and_then(|f| f.get("title"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("Feed")
                        .to_string();

                    Some(EntryViewModel {
                        id: item.get("id")?.as_str()?.parse().ok()?,
                        title: item.get("title")?.as_str()?.to_string(),
                        feed_title,
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

    let mut groups: Vec<FeedSummaryGroupViewModel> = Vec::new();
    for entry in entries {
        if let Some(group) = groups.iter_mut().find(|g| g.feed_title == entry.feed_title) {
            group.entries.push(entry);
        } else {
            groups.push(FeedSummaryGroupViewModel {
                feed_title: entry.feed_title.clone(),
                entries: vec![entry],
            });
        }
    }

    let template = AiGroupedListTemplate { groups };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => handle_htmx_error(headers, AppError::Internal(e.to_string())),
    }
}

pub async fn list_entries_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    State(cache): State<Arc<MinifluxCache>>,
    headers: HeaderMap,
    Query(query): Query<EntriesFilterQuery>,
) -> Response {
    if query.status.as_deref() == Some("ai") {
        if !config.ai_enabled {
            return handle_htmx_error(&headers, AppError::NotFound("AI digest is disabled".to_string()));
        }
        return render_ai_entries_list(&schema, &headers, &query).await;
    }
    render_entries_list(&schema, Some(&cache), &headers, &query, config.auth_enabled).await
}

pub async fn get_entry_ai_summary_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(ai_service): State<Arc<AiService>>,
    Path(id): Path<i64>,
) -> Response {
    let gql = format!(
        r#"query {{
            entry(id: "{}") {{
                id
                title
                url
                summary
                content
            }}
        }}"#,
        id
    );

    let res = schema.execute(&gql).await;
    let entry_data = res.data.into_json().ok().and_then(|d| d.get("entry").cloned());

    let (title, url, teaser) = match entry_data {
        Some(item) if !item.is_null() => {
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let summary = item.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let teaser = if !summary.is_empty() { summary } else { content }.to_string();
            (title, url, teaser)
        }
        _ => {
            let fallback_card = r#"<div class="ai-summary-card text-xs text-muted-foreground italic mt-3 pt-3 border-t border-border font-sans">Summary unavailable</div>"#;
            return Html(fallback_card.to_string()).into_response();
        }
    };

    let summary = ai_service.get_or_create_summary(id, &url, &title, &teaser).await;

    let template = AiSummaryCardTemplate { summary };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => {
            let fallback_card = r#"<div class="ai-summary-card text-xs text-muted-foreground italic mt-3 pt-3 border-t border-border font-sans">Summary unavailable</div>"#;
            Html(fallback_card.to_string()).into_response()
        }
    }
}

pub async fn mark_page_read_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    State(cache): State<Arc<MinifluxCache>>,
    headers: HeaderMap,
    Form(form): Form<MarkPageReadForm>,
) -> Response {
    if !config.auth_enabled {
        return (StatusCode::FORBIDDEN, "Mutations require authentication").into_response();
    }

    let ids: Vec<String> = if let Some(raw) = &form.entry_ids {
        if raw.trim().starts_with('[') {
            serde_json::from_str::<Vec<serde_json::Value>>(raw)
                .map(|arr| {
                    arr.into_iter()
                        .filter_map(|v| {
                            if let Some(s) = v.as_str() {
                                Some(s.to_string())
                            } else {
                                v.as_i64().map(|n| n.to_string())
                            }
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            raw.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        }
    } else {
        Vec::new()
    };

    if config.auth_enabled && !ids.is_empty() {
        let ids_json = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());
        let gql = format!(
            r#"mutation {{
                updateEntriesStatus(entryIds: {ids_json}, status: READ)
            }}"#
        );
        let res = schema.execute(&gql).await;
        if !res.errors.is_empty() {
            return handle_htmx_error(&headers, AppError::Upstream(format!("{:?}", res.errors)));
        }
    }

    cache.invalidate("entries:").await;
    cache.invalidate("index_page_entries").await;

    let filter_query = EntriesFilterQuery {
        status: form.status,
        search: form.search,
        feed_id: form.feed_id,
        category_id: form.category_id,
        limit: Some(50),
        offset: Some(0),
    };

    render_entries_list(&schema, None, &headers, &filter_query, config.auth_enabled).await
}

pub async fn get_entry_detail_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    State(cache): State<Arc<MinifluxCache>>,
    _headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    let cache_key = format!("entry:{}", id);
    let data = if let Some(cached) = cache.get(&cache_key).await {
        cached
    } else {
        let gql = format!(
            r#"query {{
                entry(id: "{id}") {{
                    id
                    title
                    summary
                    content
                    publishedAt
                    readingTime
                    status
                    isBookmarked
                    url
                    author
                    feed {{
                        title
                    }}
                }}
            }}"#
        );

        let res = schema.execute(&gql).await;
        if !res.errors.is_empty() {
            let error_card = ErrorCardTemplate {
                message: format!("Could not load article: {:?}", res.errors),
                retry_url: format!("/hx/entries/{}", id),
                target: "#detail-panel".to_string(),
            };
            return Html(error_card.render().unwrap_or_default()).into_response();
        }

        let d = match res.data.into_json() {
            Ok(d) => d,
            Err(e) => {
                let error_card = ErrorCardTemplate {
                    message: format!("Internal parsing error: {}", e),
                    retry_url: format!("/hx/entries/{}", id),
                    target: "#detail-panel".to_string(),
                };
                return Html(error_card.render().unwrap_or_default()).into_response();
            }
        };
        cache.insert_with_ttl(cache_key, d.clone(), Duration::from_secs(300)).await;
        d
    };

    let item = data.get("entry").and_then(|v| {
        if v.is_null() {
            None
        } else {
            Some(v.clone())
        }
    });

    let item = match item {
        Some(i) => i,
        None => {
            let error_card = ErrorCardTemplate {
                message: format!("Article {} could not be found or has been removed upstream.", id),
                retry_url: format!("/hx/entries/{}", id),
                target: "#detail-panel".to_string(),
            };
            return Html(error_card.render().unwrap_or_default()).into_response();
        }
    };

    let is_unread = item.get("status").and_then(|v| v.as_str()) != Some("READ");

    let mut unread_count_val: Option<i64> = None;

    if is_unread && config.auth_enabled {
        let mutation = format!(
            r#"mutation {{
                updateEntriesStatus(entryIds: ["{id}"], status: READ)
            }}"#
        );
        let _ = schema.execute(&mutation).await;
        // Fetch updated unread counter
        let counter_res = schema.execute("{ unreadCounter }").await;
        if let Some(c) = counter_res.data.into_json().ok().and_then(|d| d.get("unreadCounter").and_then(|v| v.as_i64())) {
            unread_count_val = Some(c);
        }
    }

    let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let sanitized_content = sanitize_html(content);
    let summary = item.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let feed_title = item
        .get("feed")
        .and_then(|f| f.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("Feed")
        .to_string();

    let entry = EntryViewModel {
        id,
        title: item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        feed_title,
        published_at: item.get("publishedAt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        reading_time: item.get("readingTime").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
        summary: summary.clone(),
        url: item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        author: item.get("author").and_then(|v| v.as_str()).map(|s| s.to_string()),
        is_read: true,
        is_bookmarked: item.get("isBookmarked").and_then(|v| v.as_bool()).unwrap_or(false),
    };

    let template = EntryDetailTemplate {
        entry,
        summary,
        sanitized_content,
        auth_enabled: config.auth_enabled,
    };

    let mut html = match template.render() {
        Ok(h) => h,
        Err(e) => {
            let error_card = ErrorCardTemplate {
                message: format!("Template rendering error: {}", e),
                retry_url: format!("/hx/entries/{}", id),
                target: "#detail-panel".to_string(),
            };
            return Html(error_card.render().unwrap_or_default()).into_response();
        }
    };

    if is_unread {
        if let Some(count) = unread_count_val {
            let oob_unread_pill = format!(
                r#"<span id="unread-pill" hx-swap-oob="true" class="px-2.5 py-0.5 rounded-full bg-muted border border-border text-foreground font-bold text-xs tabular">{} unread</span>"#,
                count
            );
            let oob_filter_unread_count = format!(
                r#"<span id="filter-unread-count" hx-swap-oob="true" class="text-xs tabular">({})</span>"#,
                count
            );
            html.push_str(&oob_unread_pill);
            html.push_str(&oob_filter_unread_count);
        }
        let oob_dot = format!(
            r#"<span id="entry-{}-dot" hx-swap-oob="true"></span>"#,
            id
        );
        let oob_bar = format!(
            r#"<div id="entry-{}-bar" hx-swap-oob="true" class="w-[3px] self-stretch rounded-full shrink-0 bg-transparent"></div>"#,
            id
        );
        html.push_str(&oob_dot);
        html.push_str(&oob_bar);
    }

    Html(html).into_response()
}

pub async fn toggle_bookmark_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if !config.auth_enabled {
        return (StatusCode::FORBIDDEN, "Mutations require authentication").into_response();
    }

    let gql = format!(r#"mutation {{ toggleBookmark(id: "{id}") }}"#);
    let res = schema.execute(&gql).await;
    if !res.errors.is_empty() {
        return handle_htmx_error(&headers, AppError::Upstream(format!("{:?}", res.errors)));
    }

    // Retrieve updated bookmark state from entry
    let query_gql = format!(r#"query {{ entry(id: "{id}") {{ isBookmarked }} }}"#);
    let query_res = schema.execute(&query_gql).await;
    let is_bookmarked = query_res
        .data
        .into_json()
        .ok()
        .and_then(|d| d.get("entry").and_then(|e| e.get("isBookmarked")).and_then(|v| v.as_bool()))
        .unwrap_or(true);

    let button_html = if is_bookmarked {
        format!(
            r#"<button hx-post="/hx/entries/{id}/bookmark" hx-target="this" hx-swap="outerHTML" class="p-2 rounded-full hover:bg-white/10 text-slate-400 hover:text-amber-300 transition-colors" title="Toggle Bookmark"><i data-lucide="bookmark" class="w-4 h-4 fill-amber-400 text-amber-400"></i></button>"#
        )
    } else {
        format!(
            r#"<button hx-post="/hx/entries/{id}/bookmark" hx-target="this" hx-swap="outerHTML" class="p-2 rounded-full hover:bg-white/10 text-slate-400 hover:text-amber-300 transition-colors" title="Toggle Bookmark"><i data-lucide="bookmark" class="w-4 h-4"></i></button>"#
        )
    };
    Html(button_html).into_response()
}

#[derive(Deserialize)]
pub struct ReadStatusQuery {
    pub status: Option<String>,
}

pub async fn mark_entry_read_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(query): Query<ReadStatusQuery>,
) -> Response {
    if !config.auth_enabled {
        return (StatusCode::FORBIDDEN, "Mutations require authentication").into_response();
    }

    let status_enum = match query.status.as_deref() {
        Some("unread") => "UNREAD",
        _ => "READ",
    };

    if config.auth_enabled {
        let gql = format!(
            r#"mutation {{
                updateEntryStatus(id: "{id}", status: {status_enum})
            }}"#
        );
        let res = schema.execute(&gql).await;
        if !res.errors.is_empty() {
            return handle_htmx_error(&headers, AppError::Upstream(format!("{:?}", res.errors)));
        }
    }

    // Retrieve updated entry details and unread counter
    let detail_gql = format!(
        r#"query {{
            entry(id: "{id}") {{
                id
                title
                summary
                publishedAt
                readingTime
                isBookmarked
                url
                author
                feed {{
                    title
                }}
            }}
            unreadCounter
        }}"#
    );
    let detail_res = schema.execute(&detail_gql).await;
    let data = detail_res.data.into_json().ok();

    let unread_count = data
        .as_ref()
        .and_then(|d| d.get("unreadCounter").and_then(|v| v.as_i64()))
        .map(|c| c as i32);

    let is_read = status_enum == "READ";
    let entry_data = data
        .as_ref()
        .and_then(|d| d.get("entry").cloned())
        .filter(|v| !v.is_null());

    let entry = if let Some(e) = entry_data {
        EntryViewModel {
            id,
            title: e.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            feed_title: e
                .get("feed")
                .and_then(|f| f.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("Feed")
                .to_string(),
            published_at: e.get("publishedAt").and_then(|v| v.as_str()).unwrap_or("Just now").to_string(),
            reading_time: e.get("readingTime").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
            summary: e.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            url: e.get("url").and_then(|v| v.as_str()).unwrap_or("#").to_string(),
            author: e.get("author").and_then(|v| v.as_str()).map(|s| s.to_string()),
            is_read,
            is_bookmarked: e.get("isBookmarked").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    } else {
        EntryViewModel {
            id,
            title: format!("Article {}", id),
            feed_title: "Feed".to_string(),
            published_at: "Just now".to_string(),
            reading_time: 2,
            summary: "Article marked status.".to_string(),
            url: "#".to_string(),
            author: None,
            is_read,
            is_bookmarked: false,
        }
    };

    let template = EntryItemTemplate {
        entry,
        unread_count,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => handle_htmx_error(&headers, AppError::Internal(e.to_string())),
    }
}

pub async fn mark_feed_read_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if !config.auth_enabled {
        return (StatusCode::FORBIDDEN, "Mutations require authentication").into_response();
    }

    let gql = format!(r#"mutation {{ markFeedAsRead(feedId: "{id}") }}"#);
    let res = schema.execute(&gql).await;
    if !res.errors.is_empty() {
        return handle_htmx_error(&headers, AppError::Upstream(format!("{:?}", res.errors)));
    }

    // Retrieve updated unread counter
    let counter_res = schema.execute("{ unreadCounter }").await;
    let unread_count = counter_res
        .data
        .into_json()
        .ok()
        .and_then(|d| d.get("unreadCounter").and_then(|v| v.as_i64()))
        .map(|c| c as i32);

    let template = EntryListTemplate {
        entries: Vec::new(),
        unread_count,
        current_page: 1,
        total_pages: 1,
        has_prev: false,
        has_next: false,
        prev_offset: 0,
        next_offset: 0,
        status: "all".to_string(),
        search: "".to_string(),
        limit: 50,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => handle_htmx_error(&headers, AppError::Internal(e.to_string())),
    }
}

pub async fn mark_all_read_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    headers: HeaderMap,
) -> Response {
    if !config.auth_enabled {
        return (StatusCode::FORBIDDEN, "Mutations require authentication").into_response();
    }

    let gql = "mutation { markAllAsRead }";
    let res = schema.execute(gql).await;
    if !res.errors.is_empty() {
        return handle_htmx_error(&headers, AppError::Upstream(format!("{:?}", res.errors)));
    }

    let template = EntryListTemplate {
        entries: Vec::new(),
        unread_count: Some(0),
        current_page: 1,
        total_pages: 1,
        has_prev: false,
        has_next: false,
        prev_offset: 0,
        next_offset: 0,
        status: "all".to_string(),
        search: "".to_string(),
        limit: 50,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => handle_htmx_error(&headers, AppError::Internal(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entries_filter_query_mapping() {
        // Default / unread (auth_enabled = true)
        let q_default = EntriesFilterQuery::default();
        let f_default = build_graphql_filter(&q_default, 50, 0, true);
        assert!(f_default.contains("status: UNREAD"));
        assert!(!f_default.contains("starred:"));
        assert!(!f_default.contains("publishedAfter:"));

        // Default / unread (auth_enabled = false -> guest mode: no status filter sent to Miniflux)
        let f_default_guest = build_graphql_filter(&q_default, 50, 0, false);
        assert!(!f_default_guest.contains("status:"));

        // status=unread (auth_enabled = true)
        let q_unread = EntriesFilterQuery {
            status: Some("unread".to_string()),
            ..Default::default()
        };
        let f_unread = build_graphql_filter(&q_unread, 50, 0, true);
        assert!(f_unread.contains("status: UNREAD"));

        // status=unread (auth_enabled = false -> guest mode)
        let f_unread_guest = build_graphql_filter(&q_unread, 50, 0, false);
        assert!(!f_unread_guest.contains("status:"));

        // status=read
        let q_read = EntriesFilterQuery {
            status: Some("read".to_string()),
            ..Default::default()
        };
        let f_read = build_graphql_filter(&q_read, 50, 0, true);
        assert!(f_read.contains("status: READ"));

        // status=starred
        let q_starred = EntriesFilterQuery {
            status: Some("starred".to_string()),
            ..Default::default()
        };
        let f_starred = build_graphql_filter(&q_starred, 50, 0, true);
        assert!(f_starred.contains("starred: true"));
        assert!(!f_starred.contains("status:"));

        // status=all
        let q_all = EntriesFilterQuery {
            status: Some("all".to_string()),
            ..Default::default()
        };
        let f_all = build_graphql_filter(&q_all, 50, 0, true);
        assert!(!f_all.contains("status:"));
        assert!(!f_all.contains("starred:"));
        assert!(!f_all.contains("publishedAfter:"));

        // status=today -> User decision 1B: publishedAfter only, both read and unread
        let q_today = EntriesFilterQuery {
            status: Some("today".to_string()),
            ..Default::default()
        };
        let f_today = build_graphql_filter(&q_today, 50, 0, true);
        assert!(f_today.contains("publishedAfter: "));
        assert!(!f_today.contains("status:"));

        // search keyword with quotes, backslashes, and newlines
        let q_search = EntriesFilterQuery {
            search: Some("Rust \"async\" \\ test\nnewline".to_string()),
            ..Default::default()
        };
        let f_search = build_graphql_filter(&q_search, 20, 10, true);
        assert!(f_search.contains(r#"search: "Rust \"async\" \\ test\nnewline""#));
        assert!(f_search.contains("limit: 20"));
        assert!(f_search.contains("offset: 10"));

        // empty / whitespace search is ignored
        let q_empty_search = EntriesFilterQuery {
            search: Some("   ".to_string()),
            ..Default::default()
        };
        let f_empty_search = build_graphql_filter(&q_empty_search, 50, 0, true);
        assert!(!f_empty_search.contains("search:"));
    }

    #[test]
    fn test_build_graphql_filter_search_defaults_to_all() {
        let q = EntriesFilterQuery {
            search: Some("rust".to_string()),
            status: None,
            ..Default::default()
        };
        let f = build_graphql_filter(&q, 50, 0, true);
        assert!(f.contains(r#"search: "rust""#));
        assert!(
            !f.contains("status: UNREAD") && !f.contains("status:"),
            "Search with no status must default to all articles (no status filter)"
        );
    }

    #[test]
    fn test_build_graphql_filter_guest_unread_scopes_to_today() {
        let start_of_today = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let expected_filter = format!("publishedAfter: {}", start_of_today);

        // Guest mode with explicit unread status
        let q_unread = EntriesFilterQuery {
            status: Some("unread".to_string()),
            ..Default::default()
        };
        let f_unread_guest = build_graphql_filter(&q_unread, 50, 0, false);
        assert!(
            f_unread_guest.contains(&expected_filter),
            "Guest unread filter must include publishedAfter start_of_today: {}",
            f_unread_guest
        );
        assert!(
            !f_unread_guest.contains("status:"),
            "Guest unread filter must not contain status constraint: {}",
            f_unread_guest
        );

        // Guest mode with default status (None, no search)
        let q_default = EntriesFilterQuery::default();
        let f_default_guest = build_graphql_filter(&q_default, 50, 0, false);
        assert!(
            f_default_guest.contains(&expected_filter),
            "Guest default filter must include publishedAfter start_of_today: {}",
            f_default_guest
        );
        assert!(
            !f_default_guest.contains("status:"),
            "Guest default filter must not contain status constraint: {}",
            f_default_guest
        );

        // Authenticated mode with unread status
        let f_unread_auth = build_graphql_filter(&q_unread, 50, 0, true);
        assert!(
            f_unread_auth.contains("status: UNREAD"),
            "Auth unread filter must contain status: UNREAD: {}",
            f_unread_auth
        );
        assert!(
            !f_unread_auth.contains("publishedAfter:"),
            "Auth unread filter must not restrict publishedAfter: {}",
            f_unread_auth
        );
    }
}
