use async_graphql::{Enum, InputObject, SimpleObject, ID};
use serde::{Deserialize, Serialize};

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum EntryStatus {
    #[graphql(name = "READ")]
    Read,
    #[graphql(name = "UNREAD")]
    Unread,
    #[graphql(name = "REMOVED")]
    Removed,
}

impl EntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryStatus::Read => "read",
            EntryStatus::Unread => "unread",
            EntryStatus::Removed => "removed",
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum OrderDirection {
    #[graphql(name = "ASC")]
    Asc,
    #[graphql(name = "DESC")]
    Desc,
}

impl OrderDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderDirection::Asc => "asc",
            OrderDirection::Desc => "desc",
        }
    }
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct CategoryGql {
    pub id: ID,
    pub title: String,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct FeedGql {
    pub id: ID,
    pub title: String,
    pub site_url: String,
    pub feed_url: String,
    pub checked_at: String,
    pub category: CategoryGql,
    pub unread_count: i32,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct EntryGql {
    pub id: ID,
    pub feed_id: ID,
    pub title: String,
    pub url: String,
    pub author: Option<String>,
    pub content: String,
    pub summary: String,
    pub published_at: String,
    pub status: EntryStatus,
    pub is_bookmarked: bool,
    pub reading_time: i32,
    pub feed: Option<FeedGql>,
}

#[derive(InputObject, Clone, Debug, Default)]
pub struct EntryFilterInput {
    pub status: Option<EntryStatus>,
    pub feed_id: Option<ID>,
    pub category_id: Option<ID>,
    pub starred: Option<bool>,
    pub search: Option<String>,
    pub published_after: Option<i64>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub order: Option<String>,
    pub direction: Option<OrderDirection>,
}
