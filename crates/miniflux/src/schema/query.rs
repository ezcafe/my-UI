use super::types::*;
use crate::client::{MinifluxClient, MinifluxError};
use crate::models::EntriesFilter;
use crate::summary::extract_summary;
use async_graphql::{Context, Error, Object, Result, ID};

pub struct Query;

#[Object]
impl Query {
    pub async fn entries(&self, ctx: &Context<'_>, filter: Option<EntryFilterInput>) -> Result<Vec<EntryGql>> {
        let client = ctx.data::<MinifluxClient>()?;

        let mut domain_filter = EntriesFilter::default();
        if let Some(f) = filter {
            if let Some(lim) = f.limit {
                if lim < 0 {
                    return Err(Error::new("Limit cannot be negative"));
                }
                domain_filter.limit = Some(lim as i64);
            }
            if let Some(off) = f.offset {
                if off < 0 {
                    return Err(Error::new("Offset cannot be negative"));
                }
                domain_filter.offset = Some(off as i64);
            }
            if let Some(st) = f.status {
                domain_filter.status = Some(st.as_str().to_string());
            }
            if let Some(ref fid) = f.feed_id {
                let id_val = fid.parse::<i64>().map_err(|e| Error::new(e.to_string()))?;
                domain_filter.feed_id = Some(id_val);
            }
            if let Some(ref cid) = f.category_id {
                let id_val = cid.parse::<i64>().map_err(|e| Error::new(e.to_string()))?;
                domain_filter.category_id = Some(id_val);
            }
            if let Some(st) = f.starred {
                domain_filter.starred = Some(st);
            }
            if let Some(s) = f.search {
                domain_filter.search = Some(s);
            }
            if let Some(pa) = f.published_after {
                domain_filter.published_after = Some(pa);
            }
            if let Some(o) = f.order {
                domain_filter.order = Some(o);
            }
            if let Some(d) = f.direction {
                domain_filter.direction = Some(d.as_str().to_string());
            }
        }

        let result = client.get_entries(&domain_filter).await.map_err(|e| Error::new(e.to_string()))?;

        let gql_entries = result
            .entries
            .into_iter()
            .map(|e| {
                let summary = extract_summary(&e.content, 2);
                let status = match e.status.as_str() {
                    "read" => EntryStatus::Read,
                    "removed" => EntryStatus::Removed,
                    _ => EntryStatus::Unread,
                };
                let feed = e.feed.map(|f| FeedGql {
                    id: ID(f.id.to_string()),
                    title: f.title,
                    site_url: f.site_url,
                    feed_url: f.feed_url,
                    checked_at: f.checked_at,
                    category: f.category.map(|c| CategoryGql {
                        id: ID(c.id.to_string()),
                        title: c.title,
                    }).unwrap_or(CategoryGql {
                        id: ID("0".to_string()),
                        title: "Default".to_string(),
                    }),
                    unread_count: f.unread_count,
                });

                EntryGql {
                    id: ID(e.id.to_string()),
                    feed_id: ID(e.feed_id.to_string()),
                    title: e.title,
                    url: e.url,
                    author: e.author,
                    summary,
                    content: e.content,
                    published_at: e.published_at,
                    status,
                    is_bookmarked: e.starred,
                    reading_time: e.reading_time,
                    feed,
                }
            })
            .collect();

        Ok(gql_entries)
    }

    pub async fn entry(&self, ctx: &Context<'_>, id: ID) -> Result<Option<EntryGql>> {
        let client = ctx.data::<MinifluxClient>()?;
        let entry_id = id.parse::<i64>().map_err(|e| Error::new(e.to_string()))?;
        match client.get_entry(entry_id).await {
            Ok(e) => {
                let summary = extract_summary(&e.content, 2);
                let status = match e.status.as_str() {
                    "read" => EntryStatus::Read,
                    "removed" => EntryStatus::Removed,
                    _ => EntryStatus::Unread,
                };
                let feed = e.feed.map(|f| FeedGql {
                    id: ID(f.id.to_string()),
                    title: f.title,
                    site_url: f.site_url,
                    feed_url: f.feed_url,
                    checked_at: f.checked_at,
                    category: f.category.map(|c| CategoryGql {
                        id: ID(c.id.to_string()),
                        title: c.title,
                    }).unwrap_or(CategoryGql {
                        id: ID("0".to_string()),
                        title: "Default".to_string(),
                    }),
                    unread_count: f.unread_count,
                });

                Ok(Some(EntryGql {
                    id: ID(e.id.to_string()),
                    feed_id: ID(e.feed_id.to_string()),
                    title: e.title,
                    url: e.url,
                    author: e.author,
                    summary,
                    content: e.content,
                    published_at: e.published_at,
                    status,
                    is_bookmarked: e.starred,
                    reading_time: e.reading_time,
                    feed,
                }))
            }
            Err(MinifluxError::Api { status: 404, .. }) => Ok(None),
            Err(e) => Err(Error::new(e.to_string())),
        }
    }

    pub async fn feeds(&self, ctx: &Context<'_>) -> Result<Vec<FeedGql>> {
        let client = ctx.data::<MinifluxClient>()?;
        let feeds = client.get_feeds().await.map_err(|e| Error::new(e.to_string()))?;

        let gql_feeds = feeds
            .into_iter()
            .map(|f| FeedGql {
                id: ID(f.id.to_string()),
                title: f.title,
                site_url: f.site_url,
                feed_url: f.feed_url,
                checked_at: f.checked_at,
                category: f.category.map(|c| CategoryGql {
                    id: ID(c.id.to_string()),
                    title: c.title,
                }).unwrap_or(CategoryGql {
                    id: ID("0".to_string()),
                    title: "Default".to_string(),
                }),
                unread_count: f.unread_count,
            })
            .collect();

        Ok(gql_feeds)
    }

    pub async fn categories(&self, ctx: &Context<'_>) -> Result<Vec<CategoryGql>> {
        let feeds = self.feeds(ctx).await?;
        let mut cats = Vec::new();
        for f in feeds {
            if !cats.iter().any(|c: &CategoryGql| c.id == f.category.id) {
                cats.push(f.category);
            }
        }
        Ok(cats)
    }

    pub async fn unread_counter(&self, ctx: &Context<'_>) -> Result<i32> {
        let client = ctx.data::<MinifluxClient>()?;
        let count = client.get_unread_counter().await.map_err(|e| Error::new(e.to_string()))?;
        Ok(count)
    }
}
