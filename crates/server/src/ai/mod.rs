pub mod db;
pub mod extractor;
pub mod llm;
pub mod prefetch;

use crate::config::Config;
pub use db::{AiDb, AiDbError, ArticleSummaryInput};
pub use extractor::ArticleExtractor;
pub use llm::{LlmBackend, LlmCliRunner, LlmHttpClient};
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub fn compute_page_key(status: &str, entry_ids: &[i64]) -> String {
    let mut hasher = Sha256::new();
    for id in entry_ids {
        hasher.update(id.to_le_bytes());
    }
    let hash = format!("{:x}", hasher.finalize());
    format!("{}:{}", status, &hash[..16])
}

#[derive(Clone)]
pub struct AiService {
    pub db: Arc<AiDb>,
    pub extractor: ArticleExtractor,
    pub llm: LlmBackend,
    pub in_flight: Arc<tokio::sync::Mutex<std::collections::HashSet<String>>>,
}

impl AiService {
    pub fn new(config: &Config) -> Result<Self, AiDbError> {
        let db = Arc::new(AiDb::new(&config.ai_db_path)?);
        let extractor = ArticleExtractor::new();
        let cli_runner = LlmCliRunner::new(
            config.llm_cli_cmd.clone(),
            config.llm_model.clone(),
            config.llm_extra_args.clone(),
        );
        let llm = if let Some(api_key) = &config.llm_api_key {
            let http_client = LlmHttpClient::new(
                &config.llm_provider,
                api_key,
                config.llm_api_base_url.as_deref(),
                &config.llm_model,
            );
            LlmBackend::Http {
                client: http_client,
                fallback: cli_runner,
            }
        } else {
            LlmBackend::Cli(cli_runner)
        };
        let in_flight = Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new()));
        Ok(Self {
            db,
            extractor,
            llm,
            in_flight,
        })
    }

    pub async fn get_or_create_summary(
        &self,
        entry_id: i64,
        url: &str,
        title: &str,
        teaser: &str,
    ) -> String {
        if let Some(record) = self.db.get_summary_and_title(entry_id).await {
            if record.model != "extractive-fallback" && !record.summary.is_empty() {
                return record.summary;
            }
        }

        let content = self.extractor.extract_content(url, title, teaser).await;
        let (summary, model) = self.llm.summarize(title, &content).await;

        self.db.save_summary(entry_id, &summary, &model).await.ok();
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_page_key() {
        let key1 = compute_page_key("unread", &[101, 102, 103]);
        let key2 = compute_page_key("unread", &[101, 102, 103]);
        assert_eq!(key1, key2, "same inputs produce identical key");
        assert!(key1.starts_with("unread:"), "must prefix with status: {}", key1);
        assert_eq!(key1.len(), "unread:".len() + 16, "16 hex chars after prefix");

        let key_diff_order = compute_page_key("unread", &[103, 102, 101]);
        assert_ne!(key1, key_diff_order, "different order produces different key");

        let key_diff_status = compute_page_key("all", &[101, 102, 103]);
        assert_ne!(key1, key_diff_status, "different status produces different key");
    }
}
