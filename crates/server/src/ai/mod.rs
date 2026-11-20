pub mod db;
pub mod extractor;
pub mod llm;

use crate::config::Config;
pub use db::{AiDb, AiDbError};
pub use extractor::ArticleExtractor;
pub use llm::LlmCliRunner;
use std::sync::Arc;

#[derive(Clone)]
pub struct AiService {
    pub db: Arc<AiDb>,
    pub extractor: ArticleExtractor,
    pub llm: LlmCliRunner,
}

impl AiService {
    pub fn new(config: &Config) -> Result<Self, AiDbError> {
        let db = Arc::new(AiDb::new(&config.ai_db_path)?);
        let extractor = ArticleExtractor::new();
        let llm = LlmCliRunner::new(
            config.llm_cli_cmd.clone(),
            config.llm_model.clone(),
            config.llm_extra_args.clone(),
        );
        Ok(Self { db, extractor, llm })
    }

    pub async fn get_or_create_summary(
        &self,
        entry_id: i64,
        url: &str,
        title: &str,
        teaser: &str,
    ) -> String {
        if let Some(cached) = self.db.get_summary(entry_id).await {
            return cached;
        }

        let content = self.extractor.extract_content(url, title, teaser).await;
        let (summary, model) = self.llm.summarize(title, &content).await;

        self.db.save_summary(entry_id, &summary, &model).await.ok();
        summary
    }
}
