use super::types::*;
use crate::client::MinifluxClient;
use async_graphql::{Context, Error, Object, Result, ID};

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn toggle_bookmark(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let client = ctx.data::<MinifluxClient>()?;
        let entry_id = id.parse::<i64>().map_err(|e| Error::new(e.to_string()))?;
        client
            .toggle_bookmark(entry_id)
            .await
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(true)
    }

    pub async fn update_entry_status(
        &self,
        ctx: &Context<'_>,
        id: ID,
        status: EntryStatus,
    ) -> Result<bool> {
        let client = ctx.data::<MinifluxClient>()?;
        let entry_id = id.parse::<i64>().map_err(|e| Error::new(e.to_string()))?;
        client
            .update_entry_status(vec![entry_id], status.as_str())
            .await
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(true)
    }

    pub async fn update_entries_status(
        &self,
        ctx: &Context<'_>,
        entry_ids: Vec<ID>,
        status: EntryStatus,
    ) -> Result<bool> {
        let client = ctx.data::<MinifluxClient>()?;
        let ids = entry_ids
            .into_iter()
            .map(|id| id.parse::<i64>().map_err(|e| Error::new(e.to_string())))
            .collect::<Result<Vec<i64>, _>>()?;
        client
            .update_entry_status(ids, status.as_str())
            .await
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(true)
    }

    pub async fn mark_feed_as_read(&self, ctx: &Context<'_>, feed_id: ID) -> Result<bool> {
        let client = ctx.data::<MinifluxClient>()?;
        let fid = feed_id
            .parse::<i64>()
            .map_err(|e| Error::new(e.to_string()))?;
        client
            .mark_feed_as_read(fid)
            .await
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(true)
    }

    pub async fn mark_all_as_read(&self, ctx: &Context<'_>) -> Result<bool> {
        let client = ctx.data::<MinifluxClient>()?;
        client
            .mark_all_as_read()
            .await
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(true)
    }
}
