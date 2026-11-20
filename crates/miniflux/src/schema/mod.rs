pub mod mutation;
pub mod query;
pub mod types;

use crate::client::MinifluxClient;
use async_graphql::{EmptySubscription, Schema};
pub use mutation::Mutation;
pub use query::Query;
pub use types::*;

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn build_schema(client: MinifluxClient) -> AppSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(client)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_graphql_query_entries() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 1,
                "entries": [
                    {
                        "id": 1001,
                        "feed_id": 50,
                        "title": "Exploring Spatial Audio",
                        "url": "https://example.com/audio",
                        "author": "Audio Dev",
                        "content": "<p>Spatial audio is the future. It creates immersion. Third sentence here.</p>",
                        "published_at": "2026-11-19T10:00:00Z",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 3
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "token".to_string());
        let schema = build_schema(client);

        let query = r#"
            query {
                entries(filter: { limit: 10 }) {
                    id
                    title
                    summary
                    readingTime
                    isBookmarked
                }
            }
        "#;

        let res = schema.execute(query).await;
        assert!(res.errors.is_empty(), "GraphQL execution returned errors: {:?}", res.errors);
        let data = res.data.into_json().expect("Valid json data");
        let entries = data["entries"].as_array().expect("Array of entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["id"], "1001");
        assert_eq!(entries[0]["title"], "Exploring Spatial Audio");
    }

    #[tokio::test]
    async fn test_graphql_query_feeds_includes_unread_count() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": 50,
                    "title": "Audio Weekly",
                    "site_url": "https://example.com",
                    "feed_url": "https://example.com/feed",
                    "checked_at": "2026-11-19T00:00:00Z",
                    "category": { "id": 1, "title": "Technology" }
                }
            ])))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/feeds/counters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "reads": {},
                "unreads": { "50": 12 }
            })))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "token".to_string());
        let schema = build_schema(client);

        let query = r#"
            query {
                feeds {
                    id
                    title
                    unreadCount
                    category {
                        id
                        title
                    }
                }
            }
        "#;

        let res = schema.execute(query).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().expect("Valid json data");
        let feeds = data["feeds"].as_array().expect("Feeds array");
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0]["id"], "50");
        assert_eq!(feeds[0]["unreadCount"], 12);
        assert_eq!(feeds[0]["category"]["title"], "Technology");
    }

    #[tokio::test]
    async fn test_graphql_entry_summary_resolved() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 1,
                "entries": [
                    {
                        "id": 1002,
                        "feed_id": 50,
                        "title": "Quantum Computing Update",
                        "url": "https://example.com/quantum",
                        "content": "Quantum supremacy was achieved in 2019. Now fault-tolerant logic is emerging. Commercial qubits are scaling rapidly.",
                        "published_at": "2026-11-19T00:00:00Z",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 4
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "token".to_string());
        let schema = build_schema(client);

        let query = r#"
            query {
                entries {
                    summary
                }
            }
        "#;

        let res = schema.execute(query).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().expect("Valid json data");
        let summary = data["entries"][0]["summary"].as_str().expect("Summary string");
        assert_eq!(
            summary,
            "Quantum supremacy was achieved in 2019. Now fault-tolerant logic is emerging."
        );
    }

    #[tokio::test]
    async fn test_graphql_mutation_mark_all_as_read() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/me"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 99,
                "username": "tester"
            })))
            .mount(&server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/v1/users/99/mark-all-as-read"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "token".to_string());
        let schema = build_schema(client);

        let mutation = r#"
            mutation {
                markAllAsRead
            }
        "#;

        let res = schema.execute(mutation).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().expect("Valid json data");
        assert_eq!(data["markAllAsRead"], true);
    }

    #[tokio::test]
    async fn test_graphql_invalid_filter_arguments() {
        let server = MockServer::start().await;
        let client = MinifluxClient::new(server.uri(), "token".to_string());
        let schema = build_schema(client);

        // Negative limit should be rejected with validation error
        let query = r#"
            query {
                entries(filter: { limit: -5 }) {
                    id
                }
            }
        "#;

        let res = schema.execute(query).await;
        assert!(!res.errors.is_empty(), "Expected validation error on negative limit");
    }

    #[tokio::test]
    async fn test_graphql_query_single_entry() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries/55"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 55,
                "user_id": 1,
                "feed_id": 2,
                "title": "Solo Entry 55",
                "url": "https://example.com/55",
                "content": "<p>Sentence one. Sentence two.</p>",
                "published_at": "2026-11-19T11:00:00Z",
                "status": "unread",
                "starred": false,
                "reading_time": 2,
                "feed": {
                    "id": 2,
                    "title": "Tech News",
                    "site_url": "https://tech.example.com",
                    "feed_url": "https://tech.example.com/rss",
                    "checked_at": "2026-11-19T10:00:00Z",
                    "unread_count": 5
                }
            })))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "token".to_string());
        let schema = build_schema(client);

        let query = r#"
            query {
                entry(id: "55") {
                    id
                    title
                    summary
                    readingTime
                    isBookmarked
                    feed {
                        title
                    }
                }
            }
        "#;

        let res = schema.execute(query).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().expect("Valid json");
        assert_eq!(data["entry"]["id"], "55");
        assert_eq!(data["entry"]["title"], "Solo Entry 55");
        assert_eq!(data["entry"]["feed"]["title"], "Tech News");
        assert_eq!(data["entry"]["isBookmarked"], false);
    }

    #[tokio::test]
    async fn test_graphql_mutation_mark_feed_as_read() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/v1/feeds/7/mark-all-as-read"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "token".to_string());
        let schema = build_schema(client);

        let mutation = r#"
            mutation {
                markFeedAsRead(feedId: "7")
            }
        "#;

        let res = schema.execute(mutation).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().expect("Valid json");
        assert_eq!(data["markFeedAsRead"], true);
    }

    #[tokio::test]
    async fn test_graphql_query_entries_published_after_and_search() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/entries"))
            .and(wiremock::matchers::query_param("published_after", "1732000000"))
            .and(wiremock::matchers::query_param("search", "Rust"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "total": 1,
                "entries": [
                    {
                        "id": 901,
                        "feed_id": 50,
                        "title": "Rust Weekly Roundup",
                        "url": "https://example.com/rust",
                        "author": "Rustacean",
                        "content": "<p>Today in Rust.</p>",
                        "published_at": "2026-11-19T10:00:00Z",
                        "status": "unread",
                        "starred": false,
                        "reading_time": 2
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "token".to_string());
        let schema = build_schema(client);

        let query = r#"
            query {
                entries(filter: { publishedAfter: 1732000000, search: "Rust" }) {
                    id
                    title
                }
            }
        "#;

        let res = schema.execute(query).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().expect("Valid json");
        let entries = data["entries"].as_array().expect("Array of entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["id"], "901");
        assert_eq!(entries[0]["title"], "Rust Weekly Roundup");
    }

    #[tokio::test]
    async fn test_graphql_mutation_update_entries_status() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/v1/entries"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let client = MinifluxClient::new(server.uri(), "token".to_string());
        let schema = build_schema(client);

        let mutation = r#"
            mutation {
                updateEntriesStatus(entryIds: ["101", "102"], status: READ)
            }
        "#;

        let res = schema.execute(mutation).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().expect("Valid json");
        assert_eq!(data["updateEntriesStatus"], true);
    }
}
