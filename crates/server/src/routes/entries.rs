use crate::ai::{compute_page_key, AiService};
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
    EntryListTemplate, EntryViewModel, ErrorCardTemplate, ExecutiveBriefCardTemplate,
    FeedSummaryGroupViewModel,
};

#[derive(Debug, Deserialize, Default)]
pub struct EntriesFilterQuery {
    pub status: Option<String>,
    pub feed_id: Option<i64>,
    pub category_id: Option<i64>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub request_brief: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct MarkPageReadForm {
    pub entry_ids: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub feed_id: Option<i64>,
    pub category_id: Option<i64>,
}

pub fn extract_first_image_url(html: &str) -> Option<String> {
    static IMG_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::RegexBuilder::new(r#"<img[^>]+src=["']([^"']+)["']"#)
            .case_insensitive(true)
            .build()
            .unwrap()
    });
    IMG_REGEX.captures(html).and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

pub fn is_brief_eligible_filter(status: Option<&str>, search: Option<&str>) -> bool {
    let has_search = search.is_some_and(|s| !s.trim().is_empty());
    match status {
        Some("unread") if !has_search => true,
        None if !has_search => true,
        _ => false,
    }
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
    ai_service: Option<&AiService>,
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
                            content
                            publishedAt
                            readingTime
                            status
                            isBookmarked
                            url
                            feed {{
                                title
                                category {{
                                    title
                                }}
                            }}
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
                        content
                        publishedAt
                        readingTime
                        status
                        isBookmarked
                        url
                        feed {{
                            title
                            category {{
                                title
                            }}
                        }}
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

    let mut unread_count = if auth_enabled {
        data.get("unreadCounter").and_then(|v| v.as_i64()).map(|c| c as i32)
    } else {
        data.get("todayCounter").and_then(|v| v.as_i64()).map(|c| c as i32)
    };

    let entries_result = data.get("entriesResult");
    let total = entries_result
        .and_then(|r| r.get("total"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let is_unread_status = query.status.as_deref() == Some("unread") || query.status.is_none();
    if unread_count.unwrap_or(0) == 0 && total > 0 && is_unread_status {
        unread_count = Some(total as i32);
    }

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

    let mut entries: Vec<EntryViewModel> = entries_result
        .and_then(|r| r.get("entries"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let content_str = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
                    let summary_str = item.get("summary").and_then(|v| v.as_str()).unwrap_or("");
                    let image_url = extract_first_image_url(content_str)
                        .or_else(|| extract_first_image_url(summary_str));
                    let feed_obj = item.get("feed");
                    let feed_title = feed_obj
                        .and_then(|f| f.get("title"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Feed")
                        .to_string();
                    let category_title = feed_obj
                        .and_then(|f| f.get("category"))
                        .and_then(|c| c.get("title"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    Some(EntryViewModel {
                        id: item.get("id")?.as_str()?.parse().ok()?,
                        title: item.get("title")?.as_str()?.to_string(),
                        suggested_title: None,
                        feed_title,
                        category_title,
                        published_at: item.get("publishedAt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        reading_time: item.get("readingTime").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                        summary: summary_str.to_string(),
                        image_url,
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

    let filter_eligible = is_brief_eligible_filter(query.status.as_deref(), query.search.as_deref());
    let request_brief = query.request_brief.unwrap_or(true);
    let is_eligible = filter_eligible && request_brief;

    let (ai_summaries, top_executive_brief) = if let Some(ai) = ai_service {
        let entry_ids: Vec<i64> = entries.iter().map(|e| e.id).collect();
        let summaries = ai.db.get_batch_summaries(&entry_ids).await;
        let page_brief = if filter_eligible && !is_eligible {
            ai.db.get_latest_page_brief().await
        } else {
            None
        };
        (summaries, page_brief)
    } else {
        (std::collections::HashMap::new(), None)
    };

    for entry in &mut entries {
        if let Some(record) = ai_summaries.get(&entry.id) {
            entry.suggested_title = record.suggested_title.clone();
            if !record.summary.is_empty() {
                entry.summary = record.summary.clone();
            }
        }
    }

    if let Some(ai) = ai_service {
        if is_eligible {
            let candidates: Vec<crate::ai::prefetch::PrefetchCandidate> = entries
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
            crate::ai::prefetch::AiPrefetchService::new(Arc::new(ai.clone()))
                .spawn_prefetch_job(candidates);
        }
    }

    let template = EntryListTemplate {
        entries,
        top_executive_brief,
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
        is_brief_eligible: ai_service.is_some() && is_eligible,
        offset,
        feed_id: query.feed_id,
        category_id: query.category_id,
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
                        suggested_title: None,
                        feed_title,
                        category_title: None,
                        published_at: item.get("publishedAt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        reading_time: item.get("readingTime").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
                        summary: item.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        image_url: None,
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
    State(ai_service): State<Arc<AiService>>,
    headers: HeaderMap,
    Query(query): Query<EntriesFilterQuery>,
) -> Response {
    if query.status.as_deref() == Some("ai") {
        if !config.ai_enabled {
            return handle_htmx_error(&headers, AppError::NotFound("AI digest is disabled".to_string()));
        }
        return render_ai_entries_list(&schema, &headers, &query).await;
    }
    let ai_ref = if config.ai_enabled { Some(ai_service.as_ref()) } else { None };
    render_entries_list(&schema, Some(&cache), ai_ref, &headers, &query, config.auth_enabled).await
}

pub fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

pub fn render_title_oob_span(id: i64, original_title: &str, suggested_title: &str) -> String {
    format!(
        r#"<span id="entry-title-{}" hx-swap-oob="outerHTML" class="article-title text-sm font-medium font-sans leading-snug line-clamp-2 text-foreground group-hover:text-foreground/80 transition-colors" title="{}">{}</span>"#,
        id,
        escape_html(original_title),
        escape_html(suggested_title)
    )
}

pub async fn executive_brief_htmx(
    Query(query): Query<EntriesFilterQuery>,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    State(ai_service): State<Arc<AiService>>,
) -> Response {
    let headers = [
        (axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8"),
        (axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
    ];

    // 1. Guard: Check is_brief_eligible_filter and config.ai_enabled
    if !config.ai_enabled || !is_brief_eligible_filter(query.status.as_deref(), query.search.as_deref()) {
        return (StatusCode::OK, headers, Html("")).into_response();
    }

    let limit_val = query.limit.unwrap_or(15).clamp(1, 100);
    let offset_val = query.offset.unwrap_or(0).max(0);
    let filter = build_graphql_filter(&query, limit_val, offset_val, config.auth_enabled);

    let gql = format!(
        "query {{
            entriesResult(filter: {{ {} }}) {{
                entries {{
                    id
                    title
                    summary
                    url
                    publishedAt
                    feed {{
                        title
                    }}
                }}
            }}
        }}",
        filter
    );
    let res = schema.execute(&gql).await;
    let data = match res.data.into_json() {
        Ok(d) => d,
        Err(_) => return (StatusCode::OK, headers, Html(r#"<div id="page-executive-brief" class="hidden"></div>"#)).into_response(),
    };

    struct PageBriefEntry {
        id: i64,
        title: String,
        summary: String,
        url: String,
        published_at: String,
        feed_title: String,
    }

    let entries: Vec<PageBriefEntry> = data
        .get("entriesResult")
        .and_then(|r| r.get("entries"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let id: i64 = item.get("id")?.as_str()?.parse().ok()?;
                    let title = item.get("title")?.as_str()?.to_string();
                    let summary = item.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let published_at = item.get("publishedAt").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let feed_title = item
                        .get("feed")
                        .and_then(|f| f.get("title"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("General")
                        .to_string();
                    Some(PageBriefEntry {
                        id,
                        title,
                        summary,
                        url,
                        published_at,
                        feed_title,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // 2. Empty Entries Guard
    if entries.is_empty() {
        return (StatusCode::OK, headers, Html("")).into_response();
    }

    let entry_ids: Vec<i64> = entries.iter().map(|e| e.id).collect();
    let status_str = query.status.as_deref().unwrap_or("unread");
    let page_key = compute_page_key(status_str, &entry_ids);

    let cached_map = ai_service.db.get_batch_summaries(&entry_ids).await;

    // 3. Fast-path: Check page brief cache
    if let Some(brief) = ai_service.db.get_page_brief_by_key(&page_key).await {
        let card_html = ExecutiveBriefCardTemplate { brief: &brief }
            .render()
            .unwrap_or_default();
        let mut full_html = card_html;
        for entry in &entries {
            if let Some(rec) = cached_map.get(&entry.id) {
                if let Some(suggested) = &rec.suggested_title {
                    full_html.push_str(&render_title_oob_span(entry.id, &entry.title, suggested));
                }
            }
        }
        return (StatusCode::OK, headers, Html(full_html)).into_response();
    }

    // 4. In-flight deduplication
    {
        let mut in_flight = ai_service.in_flight.lock().await;
        if in_flight.contains(&page_key) {
            drop(in_flight);
            for _ in 0..40 {
                tokio::time::sleep(Duration::from_millis(100)).await;
                if let Some(brief) = ai_service.db.get_page_brief_by_key(&page_key).await {
                    let cached_map = ai_service.db.get_batch_summaries(&entry_ids).await;
                    let card_html = ExecutiveBriefCardTemplate { brief: &brief }
                        .render()
                        .unwrap_or_default();
                    let mut full_html = card_html;
                    for entry in &entries {
                        if let Some(rec) = cached_map.get(&entry.id) {
                            if let Some(suggested) = &rec.suggested_title {
                                full_html.push_str(&render_title_oob_span(entry.id, &entry.title, suggested));
                            }
                        }
                    }
                    return (StatusCode::OK, headers, Html(full_html)).into_response();
                }
            }
            return (StatusCode::OK, headers, Html(r#"<div id="page-executive-brief" class="hidden"></div>"#)).into_response();
        }
        in_flight.insert(page_key.clone());
    }

    struct InFlightGuard {
        in_flight: Arc<tokio::sync::Mutex<std::collections::HashSet<String>>>,
        key: String,
    }
    impl Drop for InFlightGuard {
        fn drop(&mut self) {
            let in_flight = self.in_flight.clone();
            let key = self.key.clone();
            tokio::spawn(async move {
                in_flight.lock().await.remove(&key);
            });
        }
    }
    let _guard = InFlightGuard {
        in_flight: ai_service.in_flight.clone(),
        key: page_key.clone(),
    };

    // 5. Uncached extraction and batch synthesis
    let uncached_entries: Vec<&PageBriefEntry> = entries
        .iter()
        .filter(|e| {
            match cached_map.get(&e.id) {
                Some(rec) => rec.model == "extractive-fallback",
                None => true,
            }
        })
        .collect();

    let mut join_set = tokio::task::JoinSet::new();
    for e in &uncached_entries {
        let extractor = ai_service.extractor.clone();
        let id = e.id;
        let title = e.title.clone();
        let url = e.url.clone();
        let summary = e.summary.clone();
        join_set.spawn(async move {
            let content = extractor.extract_content(&url, &title, &summary).await;
            (id, content)
        });
    }

    let mut extracted_map = std::collections::HashMap::new();
    while let Some(res) = join_set.join_next().await {
        if let Ok((id, content)) = res {
            extracted_map.insert(id, content);
        }
    }

    let batch_inputs: Vec<crate::ai::llm::BatchArticleInput> = entries
        .iter()
        .map(|e| {
            let is_cached = cached_map
                .get(&e.id)
                .is_some_and(|r| r.model != "extractive-fallback");
            let content_or_brief = if is_cached {
                cached_map.get(&e.id).unwrap().summary.as_str()
            } else {
                extracted_map
                    .get(&e.id)
                    .map(|s| s.as_str())
                    .unwrap_or(e.summary.as_str())
            };
            crate::ai::llm::BatchArticleInput {
                id: e.id,
                title: e.title.as_str(),
                source: e.feed_title.as_str(),
                published_at: e.published_at.as_str(),
                content_or_brief,
                is_cached,
            }
        })
        .collect();

    let (batch_res, model) = ai_service.llm.summarize_batch_structured(&batch_inputs).await;

    // Failure contract
    if batch_res.top.is_empty() && batch_res.items.is_empty() {
        return (StatusCode::OK, headers, Html(r#"<div id="page-executive-brief" class="hidden"></div>"#)).into_response();
    }

    let mut summary_inputs = Vec::new();
    for item in &batch_res.items {
        let is_already_cached = cached_map
            .get(&item.id)
            .is_some_and(|r| r.model != "extractive-fallback");
        if !is_already_cached {
            summary_inputs.push(crate::ai::db::ArticleSummaryInput {
                entry_id: item.id,
                summary: item.brief.clone(),
                suggested_title: if item.suggested_title.is_empty() {
                    None
                } else {
                    Some(item.suggested_title.clone())
                },
                model: model.clone(),
            });
        }
    }

    let brief_text = if !batch_res.top.is_empty() {
        Some((page_key.as_str(), batch_res.top.as_str()))
    } else {
        None
    };

    if ai_service.db.save_batch_results(brief_text, &summary_inputs).await.is_err() {
        return (StatusCode::OK, headers, Html(r#"<div id="page-executive-brief" class="hidden"></div>"#)).into_response();
    }

    let card_html = if !batch_res.top.is_empty() {
        ExecutiveBriefCardTemplate { brief: &batch_res.top }
            .render()
            .unwrap_or_default()
    } else {
        r#"<div id="page-executive-brief" class="hidden"></div>"#.to_string()
    };

    let mut full_html = card_html;
    for entry in &entries {
        let suggested = batch_res
            .items
            .iter()
            .find(|i| i.id == entry.id && !i.suggested_title.is_empty())
            .map(|i| i.suggested_title.as_str())
            .or_else(|| {
                cached_map.get(&entry.id).and_then(|r| r.suggested_title.as_deref())
            });
        if let Some(sug) = suggested {
            full_html.push_str(&render_title_oob_span(entry.id, &entry.title, sug));
        }
    }

    (StatusCode::OK, headers, Html(full_html)).into_response()
}

pub async fn get_entry_ai_summary_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    State(ai_service): State<Arc<AiService>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if !config.ai_enabled {
        return handle_htmx_error(&headers, AppError::NotFound("AI digest is disabled".to_string()));
    }

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
        request_brief: None,
    };

    render_entries_list(&schema, None, None, &headers, &filter_query, config.auth_enabled).await
}

pub async fn get_entry_detail_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    State(cache): State<Arc<MinifluxCache>>,
    State(ai_service): State<Arc<AiService>>,
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
            eprintln!("Upstream GraphQL error for entry {}: {:?}", id, res.errors);
            let error_card = ErrorCardTemplate {
                message: "Could not load article. Please try again later.".to_string(),
                retry_url: format!("/hx/entries/{}", id),
                target: "#detail-panel".to_string(),
            };
            return Html(error_card.render().unwrap_or_default()).into_response();
        }

        let d = match res.data.into_json() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Internal parsing error for entry {}: {}", id, e);
                let error_card = ErrorCardTemplate {
                    message: "Failed to parse article data. Please try again.".to_string(),
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
                message: "Article could not be found or has been removed.".to_string(),
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

    let teaser_str = if !summary.is_empty() { summary.as_str() } else { content };

    let (suggested_title, summary) = if config.ai_enabled {
        if let Some(record) = ai_service.db.get_summary_and_title(id).await {
            (record.suggested_title, record.summary)
        } else {
            let fallback_teaser = if !teaser_str.is_empty() {
                teaser_str.to_string()
            } else {
                summary
            };
            (None, fallback_teaser)
        }
    } else {
        (None, summary)
    };

    let entry = EntryViewModel {
        id,
        title: item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        suggested_title,
        feed_title,
        category_title: None,
        published_at: item.get("publishedAt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        reading_time: item.get("readingTime").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
        summary: summary.clone(),
        image_url: None,
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
            eprintln!("Template rendering error for entry {}: {}", id, e);
            let error_card = ErrorCardTemplate {
                message: "Failed to display article. Please try again.".to_string(),
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
            r#"<span id="entry-{}-dot" class="inline-flex shrink-0 empty:hidden" hx-swap-oob="true"></span>"#,
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
            suggested_title: None,
            feed_title: e
                .get("feed")
                .and_then(|f| f.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("Feed")
                .to_string(),
            published_at: e.get("publishedAt").and_then(|v| v.as_str()).unwrap_or("Just now").to_string(),
            category_title: None,
            reading_time: e.get("readingTime").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
            summary: e.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            image_url: None,
            url: e.get("url").and_then(|v| v.as_str()).unwrap_or("#").to_string(),
            author: e.get("author").and_then(|v| v.as_str()).map(|s| s.to_string()),
            is_read,
            is_bookmarked: e.get("isBookmarked").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    } else {
        EntryViewModel {
            id,
            title: format!("Article {}", id),
            suggested_title: None,
            feed_title: "Feed".to_string(),
            category_title: None,
            published_at: "Just now".to_string(),
            reading_time: 2,
            summary: "Article marked status.".to_string(),
            image_url: None,
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
        top_executive_brief: None,
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
        is_brief_eligible: false,
        offset: 0,
        feed_id: None,
        category_id: None,
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
        top_executive_brief: None,
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
        is_brief_eligible: false,
        offset: 0,
        feed_id: None,
        category_id: None,
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

    #[test]
    fn test_filter_specific_brief() {
        // Only Unread and Default (None) views with no search are eligible
        assert!(is_brief_eligible_filter(Some("unread"), None));
        assert!(is_brief_eligible_filter(None, None));

        // Prohibited: today, read, starred, all, and ANY view with active search
        assert!(!is_brief_eligible_filter(Some("today"), None), "today view is prohibited from executive brief");
        assert!(!is_brief_eligible_filter(Some("unread"), Some("search")), "unread with active search is prohibited");
        assert!(!is_brief_eligible_filter(Some("today"), Some("search")), "today with active search is prohibited");
        assert!(!is_brief_eligible_filter(Some("read"), None));
        assert!(!is_brief_eligible_filter(Some("starred"), None));
        assert!(!is_brief_eligible_filter(Some("all"), None));
        assert!(!is_brief_eligible_filter(None, Some("search")));
        assert!(!is_brief_eligible_filter(Some("read"), Some("search")));
        assert!(!is_brief_eligible_filter(Some("starred"), Some("search")));
        assert!(!is_brief_eligible_filter(Some("all"), Some("search")));
    }

    #[test]
    fn test_escape_html_entities() {
        let raw = r#"<script>alert("XSS & danger")</script> 'test'"#;
        let escaped = escape_html(raw);
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
        assert!(!escaped.contains('"'));
        assert!(!escaped.contains('\''));
        assert!(escaped.contains("&lt;script&gt;"));
        assert!(escaped.contains("&quot;XSS &amp; danger&quot;"));
        assert!(escaped.contains("&#x27;test&#x27;"));
    }

    #[test]
    fn test_render_title_oob_span_typography_and_escaping() {
        let span = render_title_oob_span(
            789,
            r#"Original <script>alert(1)</script> "Quotes""#,
            r#"AI & "Punchy" Title"#,
        );
        assert!(span.contains("id=\"entry-title-789\""));
        assert!(span.contains("hx-swap-oob=\"outerHTML\""));
        assert!(span.contains("class=\"article-title text-sm font-medium font-sans leading-snug line-clamp-2 text-foreground group-hover:text-foreground/80 transition-colors\""));
        assert!(span.contains(r#"title="Original &lt;script&gt;alert(1)&lt;/script&gt; &quot;Quotes&quot;""#));
        assert!(span.contains("AI &amp; &quot;Punchy&quot; Title"));
    }

    #[test]
    fn test_executive_brief_failure_contract_hidden_div() {
        let hidden_div = r#"<div id="page-executive-brief" class="hidden"></div>"#;
        assert!(hidden_div.contains("id=\"page-executive-brief\""));
        assert!(hidden_div.contains("class=\"hidden\""));
    }

    #[tokio::test]
    async fn test_executive_brief_synthesizes_from_cached_summaries_when_uncached_empty() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use miniflux_client::client::MinifluxClient;
        use miniflux_client::schema::build_schema;
        use http_body_util::BodyExt;

        let mock_server = MockServer::start().await;
        // Mock entries endpoint returning entry 888
        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total": 1,
                "entries": [{
                    "id": 888,
                    "user_id": 1,
                    "feed_id": 10,
                    "title": "Cached Item Headline",
                    "url": "http://example.com/888",
                    "comments_url": "",
                    "author": "Author",
                    "content": "Full article content",
                    "status": "unread",
                    "starred": false,
                    "reading_time": 2,
                    "created_at": "2026-11-22T08:00:00Z",
                    "published_at": "2026-11-22T08:00:00Z",
                    "feed": {
                        "id": 10,
                        "user_id": 1,
                        "title": "Tech Feed",
                        "checked_at": "2026-11-22T08:00:00Z",
                        "category": { "id": 1, "user_id": 1, "title": "Tech" }
                    }
                }]
            })))
            .mount(&mock_server)
            .await;

        let db_file = tempfile::NamedTempFile::new().unwrap();
        let db = Arc::new(crate::ai::AiDb::new(db_file.path().to_str().unwrap()).unwrap());
        // Save summary in DB for entry 888 beforehand so uncached_set will be empty!
        db.save_summary_and_title(888, Some("Punchy 888 Title"), "Cached item summary of 888", "test-model")
            .await
            .unwrap();

        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        let script_content = "#!/bin/sh\nprintf '%s' '{\"top\":\"Synthesized From Cached Brief\",\"items\":[]}'\n";
        script_file.write_all(script_content.as_bytes()).unwrap();
        let mut perms = std::fs::metadata(script_file.path()).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(script_file.path(), perms).unwrap();

        let llm = crate::ai::llm::LlmCliRunner::new(
            script_file.path().to_str().unwrap().to_string(),
            "test-model".to_string(),
            vec![],
        );

        let ai_service = Arc::new(crate::ai::AiService {
            db: db.clone(),
            extractor: crate::ai::ArticleExtractor::new(),
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        });

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
            miniflux_base_url: mock_server.uri(),
            miniflux_api_key: "api-key".to_string(),
            ai_enabled: true,
            llm_cli_cmd: "llm".to_string(),
            llm_model: "test-model".to_string(),
            llm_extra_args: vec![],
            ai_db_path: db_file.path().to_str().unwrap().to_string(),
            llm_provider: "openai".to_string(),
            llm_api_key: None,
            llm_api_base_url: None,
        });

        let client = MinifluxClient::new(mock_server.uri(), "api-key".to_string());
        let schema = build_schema(client);

        let query = EntriesFilterQuery {
            status: Some("unread".to_string()),
            limit: Some(15),
            offset: Some(0),
            feed_id: None,
            category_id: None,
            search: None,
            request_brief: None,
        };

        let response = executive_brief_htmx(
            Query(query),
            State(schema),
            State(config),
            State(ai_service.clone()),
        ).await;

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Must NOT be the empty hidden div
        assert!(!html.contains("class=\"hidden\""), "Must not return hidden div when entries exist: {}", html);
        assert!(html.contains("Synthesized From Cached Brief"));
        assert!(html.contains("id=\"entry-title-888\""));

        // Verify saved in DB with page_key
        let page_key = crate::ai::compute_page_key("unread", &[888]);
        let saved_brief = db.get_page_brief_by_key(&page_key).await;
        assert_eq!(saved_brief.as_deref(), Some("Synthesized From Cached Brief"));
    }

    #[tokio::test]
    async fn test_executive_brief_reuses_db_summaries_and_synthesizes_top() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use miniflux_client::client::MinifluxClient;
        use miniflux_client::schema::build_schema;
        use http_body_util::BodyExt;

        let mock_server = MockServer::start().await;
        // Mock entries endpoint returning entry 888 and 999
        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total": 2,
                "entries": [
                    {
                        "id": 888,
                        "user_id": 1,
                        "feed_id": 10,
                        "title": "Cached Item 888",
                        "url": "http://example.com/888",
                        "comments_url": "",
                        "author": "Author A",
                        "content": "Content 888",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 2,
                        "created_at": "2026-11-22T08:00:00Z",
                        "published_at": "2026-11-22T08:00:00Z",
                        "feed": {
                            "id": 10,
                            "user_id": 1,
                            "title": "Tech Feed",
                            "checked_at": "2026-11-22T08:00:00Z",
                            "category": { "id": 1, "user_id": 1, "title": "Tech" }
                        }
                    },
                    {
                        "id": 999,
                        "user_id": 1,
                        "feed_id": 10,
                        "title": "New Item 999",
                        "url": "http://example.com/999",
                        "comments_url": "",
                        "author": "Author B",
                        "content": "New full content of 999",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 3,
                        "created_at": "2026-11-22T09:00:00Z",
                        "published_at": "2026-11-22T09:00:00Z",
                        "feed": {
                            "id": 10,
                            "user_id": 1,
                            "title": "Tech Feed",
                            "checked_at": "2026-11-22T09:00:00Z",
                            "category": { "id": 1, "user_id": 1, "title": "Tech" }
                        }
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let db_file = tempfile::NamedTempFile::new().unwrap();
        let db = Arc::new(crate::ai::AiDb::new(db_file.path().to_str().unwrap()).unwrap());
        // Save summary in DB for entry 888 beforehand
        db.save_summary_and_title(888, Some("Existing 888 Title"), "Existing brief 888", "test-model")
            .await
            .unwrap();

        let prompt_dump_file = tempfile::NamedTempFile::new().unwrap();
        let prompt_dump_path = prompt_dump_file.path().to_str().unwrap().to_string();

        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        let script_content = format!(
            "#!/bin/sh\ncat > \"{}\"\nprintf '%s' '{{\"top\":\"Whole Page Brief Synthesis\",\"items\":[{{\"i\":999,\"t\":\"New 999 Title\",\"b\":\"New brief 999 sentence 1. Sentence 2. Sentence 3.\"}}]}}'\n",
            prompt_dump_path
        );
        script_file.write_all(script_content.as_bytes()).unwrap();
        let mut perms = std::fs::metadata(script_file.path()).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(script_file.path(), perms).unwrap();

        let llm = crate::ai::llm::LlmCliRunner::new(
            script_file.path().to_str().unwrap().to_string(),
            "test-model".to_string(),
            vec![],
        );

        let ai_service = Arc::new(crate::ai::AiService {
            db: db.clone(),
            extractor: crate::ai::ArticleExtractor::new(),
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        });

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
            miniflux_base_url: mock_server.uri(),
            miniflux_api_key: "api-key".to_string(),
            ai_enabled: true,
            llm_cli_cmd: "llm".to_string(),
            llm_model: "test-model".to_string(),
            llm_extra_args: vec![],
            ai_db_path: db_file.path().to_str().unwrap().to_string(),
            llm_provider: "openai".to_string(),
            llm_api_key: None,
            llm_api_base_url: None,
        });

        let client = MinifluxClient::new(mock_server.uri(), "api-key".to_string());
        let schema = build_schema(client);

        let query = EntriesFilterQuery {
            status: Some("unread".to_string()),
            limit: Some(15),
            offset: Some(0),
            feed_id: None,
            category_id: None,
            search: None,
            request_brief: None,
        };

        let response = executive_brief_htmx(
            Query(query),
            State(schema),
            State(config),
            State(ai_service.clone()),
        ).await;

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert!(html.contains("Whole Page Brief Synthesis"), "Html should contain synthesized top: {}", html);
        assert!(html.contains("id=\"entry-title-999\""));
        assert!(html.contains("id=\"entry-title-888\""));

        // Verify the prompt piped to LLM contained BOTH existing 888 brief and new 999 article
        let piped_prompt = std::fs::read_to_string(&prompt_dump_path).unwrap_or_default();
        assert!(piped_prompt.contains("[ID 888] (Existing Brief)"), "Prompt must include cached brief for 888: {}", piped_prompt);
        assert!(piped_prompt.contains("[ID 999] (New Article)"), "Prompt must include new article for 999: {}", piped_prompt);
        assert!(piped_prompt.contains("### Source: Tech Feed"), "Prompt must include source header");

        // Verify 999 was saved to DB and 888 was preserved
        let summaries = db.get_batch_summaries(&[888, 999]).await;
        assert_eq!(summaries.get(&888).unwrap().summary, "Existing brief 888");
        assert!(summaries.get(&999).unwrap().summary.contains("New brief 999"));

        // Verify page executive brief was saved
        let page_key = crate::ai::compute_page_key("unread", &[888, 999]);
        let saved_brief = db.get_page_brief_by_key(&page_key).await;
        assert_eq!(saved_brief.as_deref(), Some("Whole Page Brief Synthesis"));
    }

    #[test]
    fn test_entries_filter_query_request_brief_parsing() {
        use axum::extract::Query;
        use axum::http::Uri;

        let uri: Uri = "/entries?request_brief=true".parse().unwrap();
        let Query(q): Query<EntriesFilterQuery> = Query::try_from_uri(&uri).unwrap();
        assert_eq!(q.request_brief, Some(true));

        let uri: Uri = "/entries?request_brief=false".parse().unwrap();
        let Query(q): Query<EntriesFilterQuery> = Query::try_from_uri(&uri).unwrap();
        assert_eq!(q.request_brief, Some(false));

        let uri: Uri = "/entries".parse().unwrap();
        let Query(q): Query<EntriesFilterQuery> = Query::try_from_uri(&uri).unwrap();
        assert_eq!(q.request_brief, None);
    }

    #[tokio::test]
    async fn test_unread_count_fallback_to_total_in_entries() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use miniflux_client::client::MinifluxClient;
        use miniflux_client::schema::build_schema;
        use http_body_util::BodyExt;

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total": 5,
                "entries": [{
                    "id": 100,
                    "user_id": 1,
                    "feed_id": 1,
                    "title": "Unread Article",
                    "url": "http://example.com/100",
                    "comments_url": "",
                    "author": "Author",
                    "content": "Content",
                    "status": "unread",
                    "starred": false,
                    "reading_time": 1,
                    "created_at": "2026-11-23T08:00:00Z",
                    "published_at": "2026-11-23T08:00:00Z",
                    "feed": {
                        "id": 1,
                        "user_id": 1,
                        "title": "Tech",
                        "checked_at": "2026-11-23T08:00:00Z",
                        "category": { "id": 1, "user_id": 1, "title": "Tech" }
                    }
                }]
            })))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "reads": {},
                "unreads": {}
            })))
            .mount(&mock_server)
            .await;

        let client = MinifluxClient::new(mock_server.uri(), "api-key".to_string());
        let schema = build_schema(client);
        let headers = axum::http::HeaderMap::new();
        let query = EntriesFilterQuery {
            status: Some("unread".to_string()),
            ..Default::default()
        };

        let response = render_entries_list(&schema, None, None, &headers, &query, true).await;
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert!(html.contains(r#"data-base-unread="5""#), "Must fallback unread count to total (5) when counter is 0: {}", html);
        assert!(html.contains("(5)"), "Must display (5) in unread count span: {}", html);
    }

    #[tokio::test]
    async fn test_render_entries_list_request_brief_false_loads_latest_page_brief() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use miniflux_client::client::MinifluxClient;
        use miniflux_client::schema::build_schema;
        use http_body_util::BodyExt;

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total": 1,
                "entries": [{
                    "id": 100,
                    "user_id": 1,
                    "feed_id": 1,
                    "title": "Article",
                    "url": "http://example.com/100",
                    "comments_url": "",
                    "author": "Author",
                    "content": "Content",
                    "status": "unread",
                    "starred": false,
                    "reading_time": 1,
                    "created_at": "2026-11-23T08:00:00Z",
                    "published_at": "2026-11-23T08:00:00Z",
                    "feed": {
                        "id": 1,
                        "user_id": 1,
                        "title": "Tech",
                        "checked_at": "2026-11-23T08:00:00Z",
                        "category": { "id": 1, "user_id": 1, "title": "Tech" }
                    }
                }]
            })))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "reads": {},
                "unreads": { "1": 1 }
            })))
            .mount(&mock_server)
            .await;

        let db_file = tempfile::NamedTempFile::new().unwrap();
        let db = Arc::new(crate::ai::AiDb::new(db_file.path().to_str().unwrap()).unwrap());
        db.save_page_brief("Existing Latest Executive Brief").await.unwrap();

        let llm = crate::ai::llm::LlmCliRunner::new(
            "llm".to_string(),
            "test-model".to_string(),
            vec![],
        );

        let ai_service = crate::ai::AiService {
            db: db.clone(),
            extractor: crate::ai::ArticleExtractor::new(),
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        };

        let client = MinifluxClient::new(mock_server.uri(), "api-key".to_string());
        let schema = build_schema(client);
        let headers = axum::http::HeaderMap::new();
        let query = EntriesFilterQuery {
            status: Some("unread".to_string()),
            request_brief: Some(false),
            ..Default::default()
        };

        let response = render_entries_list(&schema, None, Some(&ai_service), &headers, &query, true).await;
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert!(html.contains("Existing Latest Executive Brief"), "Must render latest saved page brief: {}", html);
        assert!(!html.contains("hx-trigger=\"load\""), "Must not trigger brief generation when request_brief is false: {}", html);
    }

    #[test]
    fn test_extract_first_image_url() {
        assert_eq!(
            extract_first_image_url(r#"<p>Some text</p><img src="https://example.com/banner.png" alt="test"/><p>more</p>"#),
            Some("https://example.com/banner.png".to_string())
        );
        assert_eq!(
            extract_first_image_url(r#"<figure><IMG class="hero" SRC='https://example.com/hero.jpg'></figure>"#),
            Some("https://example.com/hero.jpg".to_string())
        );
        assert_eq!(
            extract_first_image_url("<p>No image here</p>"),
            None
        );
        assert_eq!(
            extract_first_image_url(""),
            None
        );
    }
}
