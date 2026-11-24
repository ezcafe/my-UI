use crate::models::*;
use reqwest::header::{HeaderMap, HeaderValue};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum MinifluxError {
    #[error("Reqwest error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Client timeout")]
    Timeout,
    #[error("API error ({status}): {body}")]
    Api { status: u16, body: String },
    #[error("User not found or unauthenticated")]
    NoUser,
}

impl MinifluxError {
    fn from_reqwest(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            MinifluxError::Timeout
        } else {
            MinifluxError::Network(err)
        }
    }
}

#[derive(Clone)]
pub struct MinifluxClient {
    pub base_url: String,
    pub api_key: String,
    http_client: reqwest::Client,
    cached_user_id: Arc<RwLock<Option<i64>>>,
}

impl MinifluxClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self::with_timeout(base_url, api_key, Duration::from_secs(5))
    }

    pub fn with_timeout(base_url: String, api_key: String, timeout: Duration) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Auth-Token",
            HeaderValue::from_str(&api_key).unwrap_or(HeaderValue::from_static("")),
        );

        let http_client = reqwest::Client::builder()
            .timeout(timeout)
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            http_client,
            cached_user_id: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_me(&self) -> Result<User, MinifluxError> {
        let url = format!("{}/v1/me", self.base_url);
        let resp = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(MinifluxError::from_reqwest)?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(MinifluxError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let user: User = resp.json().await.map_err(MinifluxError::from_reqwest)?;
        *self.cached_user_id.write().await = Some(user.id);
        Ok(user)
    }

    pub async fn get_cached_user_id(&self) -> Result<i64, MinifluxError> {
        if let Some(id) = *self.cached_user_id.read().await {
            return Ok(id);
        }
        let user = self.get_me().await?;
        Ok(user.id)
    }

    pub async fn get_feeds(&self) -> Result<Vec<Feed>, MinifluxError> {
        let feeds_url = format!("{}/v1/feeds", self.base_url);
        let counters_url = format!("{}/v1/feeds/counters", self.base_url);

        let feeds_fut = self.http_client.get(&feeds_url).send();
        let counters_fut = self.http_client.get(&counters_url).send();

        let (feeds_resp, counters_resp) = tokio::join!(feeds_fut, counters_fut);

        let feeds_resp = feeds_resp.map_err(MinifluxError::from_reqwest)?;
        let counters_resp = counters_resp.map_err(MinifluxError::from_reqwest)?;

        if !feeds_resp.status().is_success() {
            let status = feeds_resp.status().as_u16();
            let body = feeds_resp.text().await.unwrap_or_default();
            return Err(MinifluxError::Api { status, body });
        }

        let mut feeds: Vec<Feed> = feeds_resp
            .json()
            .await
            .map_err(MinifluxError::from_reqwest)?;

        let counters: FeedCounters = if counters_resp.status().is_success() {
            counters_resp.json().await.unwrap_or(FeedCounters {
                reads: Default::default(),
                unreads: Default::default(),
            })
        } else {
            FeedCounters {
                reads: Default::default(),
                unreads: Default::default(),
            }
        };

        for feed in &mut feeds {
            let key = feed.id.to_string();
            if let Some(&unread) = counters.unreads.get(&key) {
                feed.unread_count = unread;
            }
        }

        Ok(feeds)
    }

    pub async fn get_entries(
        &self,
        filter: &EntriesFilter,
    ) -> Result<EntryResultSet, MinifluxError> {
        let url = format!("{}/v1/entries", self.base_url);
        let mut req = self.http_client.get(&url);

        if let Some(ref status) = filter.status {
            req = req.query(&[("status", status)]);
        }
        if let Some(feed_id) = filter.feed_id {
            req = req.query(&[("feed_id", feed_id.to_string())]);
        }
        if let Some(category_id) = filter.category_id {
            req = req.query(&[("category_id", category_id.to_string())]);
        }
        if let Some(starred) = filter.starred {
            req = req.query(&[("starred", starred.to_string())]);
        }
        if let Some(ref search) = filter.search {
            req = req.query(&[("search", search)]);
        }
        if let Some(published_after) = filter.published_after {
            req = req.query(&[("published_after", published_after.to_string())]);
        }
        if let Some(limit) = filter.limit {
            req = req.query(&[("limit", limit.to_string())]);
        }
        if let Some(offset) = filter.offset {
            req = req.query(&[("offset", offset.to_string())]);
        }
        if let Some(ref order) = filter.order {
            req = req.query(&[("order", order)]);
        }
        if let Some(ref direction) = filter.direction {
            req = req.query(&[("direction", direction)]);
        }

        let resp = req.send().await.map_err(MinifluxError::from_reqwest)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(MinifluxError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let result: EntryResultSet = resp.json().await.map_err(MinifluxError::from_reqwest)?;
        Ok(result)
    }

    pub async fn update_entry_status(
        &self,
        entry_ids: Vec<i64>,
        status: &str,
    ) -> Result<(), MinifluxError> {
        let url = format!("{}/v1/entries", self.base_url);
        let payload = EntriesStatusUpdateRequest {
            entry_ids,
            status: status.to_string(),
        };

        let resp = self
            .http_client
            .put(&url)
            .json(&payload)
            .send()
            .await
            .map_err(MinifluxError::from_reqwest)?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(MinifluxError::Api {
                status: status.as_u16(),
                body,
            });
        }

        Ok(())
    }

    pub async fn mark_all_as_read(&self) -> Result<(), MinifluxError> {
        let user_id = self.get_cached_user_id().await?;
        let url = format!("{}/v1/users/{}/mark-all-as-read", self.base_url, user_id);

        let resp = self
            .http_client
            .put(&url)
            .send()
            .await
            .map_err(MinifluxError::from_reqwest)?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(MinifluxError::Api {
                status: status.as_u16(),
                body,
            });
        }

        Ok(())
    }

    pub async fn toggle_bookmark(&self, entry_id: i64) -> Result<(), MinifluxError> {
        let url = format!("{}/v1/entries/{}/bookmark", self.base_url, entry_id);

        let resp = self
            .http_client
            .put(&url)
            .send()
            .await
            .map_err(MinifluxError::from_reqwest)?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(MinifluxError::Api {
                status: status.as_u16(),
                body,
            });
        }

        Ok(())
    }

    pub async fn get_unread_counter(&self) -> Result<i32, MinifluxError> {
        let counters_url = format!("{}/v1/feeds/counters", self.base_url);
        let resp = self
            .http_client
            .get(&counters_url)
            .send()
            .await
            .map_err(MinifluxError::from_reqwest)?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(MinifluxError::Api { status, body });
        }

        let counters: FeedCounters = resp.json().await.map_err(MinifluxError::from_reqwest)?;
        let total: i32 = counters.unreads.values().sum();
        Ok(total)
    }

    pub async fn get_entry(&self, entry_id: i64) -> Result<Entry, MinifluxError> {
        let url = format!("{}/v1/entries/{}", self.base_url, entry_id);
        let resp = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(MinifluxError::from_reqwest)?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(MinifluxError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let entry: Entry = resp.json().await.map_err(MinifluxError::from_reqwest)?;
        Ok(entry)
    }

    pub async fn mark_feed_as_read(&self, feed_id: i64) -> Result<(), MinifluxError> {
        let url = format!("{}/v1/feeds/{}/mark-all-as-read", self.base_url, feed_id);
        let resp = self
            .http_client
            .put(&url)
            .send()
            .await
            .map_err(MinifluxError::from_reqwest)?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(MinifluxError::Api {
                status: status.as_u16(),
                body,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_me_caches_user_id() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/me"))
            .and(header("X-Auth-Token", "secret-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 42,
                "username": "alice",
                "is_admin": true
            })))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "secret-token".to_string());
        let me = client.get_me().await.expect("Failed to get me");
        assert_eq!(me.id, 42);
        assert_eq!(me.username, "alice");

        let cached = client
            .get_cached_user_id()
            .await
            .expect("Expected cached id");
        assert_eq!(cached, 42);
    }

    #[tokio::test]
    async fn test_mark_all_as_read_uses_cached_user_id() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 42,
                "username": "alice"
            })))
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/v1/users/42/mark-all-as-read"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "secret-token".to_string());
        client
            .mark_all_as_read()
            .await
            .expect("Failed to mark all as read");
    }

    #[tokio::test]
    async fn test_get_feeds_merges_unread_counters() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": 10,
                    "title": "Rust Blog",
                    "site_url": "https://blog.rust-lang.org",
                    "feed_url": "https://blog.rust-lang.org/feed.xml",
                    "checked_at": "2026-11-19T00:00:00Z"
                },
                {
                    "id": 20,
                    "title": "Tech News",
                    "site_url": "https://technews.example.com",
                    "feed_url": "https://technews.example.com/rss",
                    "checked_at": "2026-11-19T00:00:00Z"
                }
            ])))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "reads": {},
                "unreads": {
                    "10": 7,
                    "20": 0
                }
            })))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "secret-token".to_string());
        let feeds = client.get_feeds().await.expect("Failed to get feeds");
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds[0].id, 10);
        assert_eq!(feeds[0].unread_count, 7);
        assert_eq!(feeds[1].id, 20);
        assert_eq!(feeds[1].unread_count, 0);
    }

    #[tokio::test]
    async fn test_get_entries_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 1,
                "entries": [
                    {
                        "id": 101,
                        "feed_id": 10,
                        "title": "Announcing Rust 1.100",
                        "url": "https://blog.rust-lang.org/2026/11/05/Rust-1.100.0.html",
                        "author": "The Rust Team",
                        "content": "<p>We are delighted to announce Rust 1.100.</p>",
                        "published_at": "2026-11-05T00:00:00Z",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 2
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "secret-token".to_string());
        let filter = EntriesFilter {
            status: Some("unread".to_string()),
            limit: Some(10),
            ..Default::default()
        };
        let res = client
            .get_entries(&filter)
            .await
            .expect("Failed to fetch entries");
        assert_eq!(res.total, 1);
        assert_eq!(res.entries.len(), 1);
        assert_eq!(res.entries[0].id, 101);
        assert_eq!(res.entries[0].title, "Announcing Rust 1.100");
    }

    #[tokio::test]
    async fn test_miniflux_client_get_entries_published_after() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .and(wiremock::matchers::query_param(
                "published_after",
                "1732000000",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 1,
                "entries": [
                    {
                        "id": 105,
                        "feed_id": 10,
                        "title": "Today's Rust Post",
                        "url": "https://example.com/today",
                        "author": "Rustacean",
                        "content": "<p>Published today.</p>",
                        "published_at": "2026-11-19T08:00:00Z",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 1
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "secret-token".to_string());
        let filter = EntriesFilter {
            published_after: Some(1732000000),
            ..Default::default()
        };
        let res = client
            .get_entries(&filter)
            .await
            .expect("Failed to fetch entries");
        assert_eq!(res.total, 1);
        assert_eq!(res.entries[0].id, 105);
        assert_eq!(res.entries[0].title, "Today's Rust Post");
    }

    #[tokio::test]
    async fn test_update_entry_status_sends_batch_payload() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "secret-token".to_string());
        client
            .update_entry_status(vec![101, 102], "read")
            .await
            .expect("Failed to update status");
    }

    #[tokio::test]
    async fn test_toggle_bookmark_mutation() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/v1/entries/101/bookmark"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "secret-token".to_string());
        client
            .toggle_bookmark(101)
            .await
            .expect("Failed to toggle bookmark");
    }

    #[tokio::test]
    async fn test_miniflux_timeout_handling() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/me"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(150))
                    .set_body_json(json!({"id": 1, "username": "slow"})),
            )
            .mount(&server)
            .await;

        let client = MinifluxClient::with_timeout(
            server.uri(),
            "secret".to_string(),
            Duration::from_millis(50),
        );
        let res = client.get_me().await;
        assert!(res.is_err(), "Expected timeout error");
        match res.unwrap_err() {
            MinifluxError::Timeout => (),
            other => panic!("Expected MinifluxError::Timeout, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_entry_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries/42"))
            .and(header("X-Auth-Token", "secret-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 42,
                "user_id": 1,
                "feed_id": 10,
                "title": "Article 42",
                "url": "https://example.com/42",
                "content": "<p>Content of 42</p>",
                "published_at": "2026-11-19T10:00:00Z",
                "status": "unread",
                "starred": true,
                "reading_time": 3
            })))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "secret-token".to_string());
        let entry = client.get_entry(42).await.expect("Failed to get entry");
        assert_eq!(entry.id, 42);
        assert_eq!(entry.title, "Article 42");
        assert!(entry.starred);
    }

    #[tokio::test]
    async fn test_mark_feed_as_read_endpoint() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/v1/feeds/5/mark-all-as-read"))
            .and(header("X-Auth-Token", "secret-token"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "secret-token".to_string());
        client
            .mark_feed_as_read(5)
            .await
            .expect("Failed to mark feed as read");
    }
}
