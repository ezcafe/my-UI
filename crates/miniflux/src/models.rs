use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Category {
    pub id: i64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedIcon {
    pub feed_id: i64,
    pub icon_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Feed {
    pub id: i64,
    #[serde(default)]
    pub user_id: i64,
    pub title: String,
    #[serde(default)]
    pub site_url: String,
    #[serde(default)]
    pub feed_url: String,
    #[serde(default)]
    pub checked_at: String,
    #[serde(default)]
    pub category: Option<Category>,
    #[serde(default)]
    pub unread_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub id: i64,
    #[serde(default)]
    pub user_id: i64,
    pub feed_id: i64,
    pub title: String,
    pub url: String,
    pub author: Option<String>,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub published_at: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub starred: bool,
    #[serde(default)]
    pub reading_time: i32,
    #[serde(default)]
    pub feed: Option<Feed>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryResultSet {
    pub total: i64,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedCounters {
    #[serde(default)]
    pub reads: HashMap<String, i32>,
    #[serde(default)]
    pub unreads: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntriesStatusUpdateRequest {
    pub entry_ids: Vec<i64>,
    pub status: String,
}

#[derive(Debug, Clone, Default)]
pub struct EntriesFilter {
    pub status: Option<String>,
    pub feed_id: Option<i64>,
    pub category_id: Option<i64>,
    pub starred: Option<bool>,
    pub search: Option<String>,
    pub published_after: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub order: Option<String>,
    pub direction: Option<String>,
}
