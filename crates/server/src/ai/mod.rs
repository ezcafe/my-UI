pub mod db;
pub mod extractor;
pub mod llm;
pub mod prefetch;

use crate::config::Config;
pub use db::{AiDb, AiDbError, ArticleSummaryInput};
pub use extractor::ArticleExtractor;
pub use llm::{DetailAnalysisResult, LlmBackend, LlmCliRunner, LlmHttpClient};
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
    pub async fn new(config: &Config) -> Result<Self, AiDbError> {
        let db = if !config.ai_enabled {
            Arc::new(AiDb::new_mock())
        } else {
            Arc::new(AiDb::new(&config.database_url).await?)
        };
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
    ) -> Result<String, String> {
        if let Some(record) = self.db.get_summary_and_title(entry_id).await {
            if record.model != "extractive-fallback" && !record.summary.is_empty() {
                return Ok(record.summary);
            }
        }

        let content = self.extractor.extract_content(url, title, teaser).await;
        let (summary, model) = self.llm.summarize(title, &content).await;

        if model == "extractive-fallback" || summary.trim().is_empty() {
            return Err("LLM summary unavailable".to_string());
        }

        self.db.save_summary(entry_id, &summary, &model).await.ok();
        Ok(summary)
    }

    pub async fn generate_rewrite_and_analysis(
        &self,
        entry_id: i64,
        title: &str,
        content: &str,
    ) -> Result<DetailAnalysisResult, String> {
        let (analysis, model) = self.llm.rewrite_and_analyze(title, content).await;
        if !analysis.rewrite.is_empty() && model != "error" {
            let existing = self.db.get_summary_and_title(entry_id).await;
            let summary = existing
                .as_ref()
                .map(|r| r.summary.clone())
                .unwrap_or_else(|| title.to_string());
            let suggested_title = existing.as_ref().and_then(|r| r.suggested_title.as_deref());

            let _ = self
                .db
                .save_summary_and_title_with_analysis(
                    entry_id,
                    suggested_title,
                    &summary,
                    Some(&analysis.rewrite),
                    Some(&analysis.impact),
                    Some(&analysis.fallacies),
                    &model,
                )
                .await;
            Ok(analysis)
        } else {
            Err("Failed to generate rewrite and analysis".to_string())
        }
    }

    pub async fn generate_rewrite(
        &self,
        entry_id: i64,
        title: &str,
        content: &str,
    ) -> Result<String, String> {
        self.generate_rewrite_and_analysis(entry_id, title, content)
            .await
            .map(|a| a.rewrite)
    }

    pub async fn analyze_entry_impact_and_fallacies(
        &self,
        entry_id: i64,
        title: &str,
        content: &str,
    ) -> Result<(String, String), String> {
        if let Some(record) = self.db.get_summary_and_title(entry_id).await {
            if record.model != "extractive-fallback" {
                let imp = record.impact.as_deref().unwrap_or("").trim();
                let fal = record.fallacies.as_deref().unwrap_or("").trim();
                if !imp.is_empty() || !fal.is_empty() {
                    return Ok((imp.to_string(), fal.to_string()));
                }
            }
        }

        let ((impact, fallacies), model) =
            self.llm.analyze_impact_and_fallacies(title, content).await;
        if (!impact.is_empty() || !fallacies.is_empty()) && model != "error" {
            let existing = self.db.get_summary_and_title(entry_id).await;
            let summary = existing
                .as_ref()
                .map(|r| r.summary.clone())
                .unwrap_or_else(|| title.to_string());
            let suggested_title = existing.as_ref().and_then(|r| r.suggested_title.as_deref());
            let rewrite = existing.as_ref().and_then(|r| r.rewrite.as_deref());

            let _ = self
                .db
                .save_summary_and_title_with_analysis(
                    entry_id,
                    suggested_title,
                    &summary,
                    rewrite,
                    Some(&impact),
                    Some(&fallacies),
                    &model,
                )
                .await;
            Ok((impact, fallacies))
        } else {
            Err("Failed to generate impact and fallacies".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_single_rewrite() {
        let db = Arc::new(AiDb::new_mock());
        let extractor = ArticleExtractor::new();

        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        let script_content = "#!/bin/sh\nprintf '%b' 'Single rewrite lead sentence. High impact outcome sentence.\\n\\n• Context point 1.'\n";
        script_file.write_all(script_content.as_bytes()).unwrap();
        let mut perms = std::fs::metadata(script_file.path()).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(script_file.path(), perms).unwrap();

        let llm = LlmCliRunner::new(
            script_file.path().to_str().unwrap().to_string(),
            "test-model".to_string(),
            vec![],
        );

        let ai = AiService {
            db: db.clone(),
            extractor,
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        };

        // Seed an initial summary first
        db.save_summary(555, "Initial brief", "test-model")
            .await
            .unwrap();

        let rewrite_res = ai
            .generate_rewrite(555, "Test Title", "Article content with details.")
            .await;
        assert!(rewrite_res.is_ok(), "generate_rewrite should succeed");
        let rewrite = rewrite_res.unwrap();
        assert!(rewrite.contains("Single rewrite lead sentence. High impact outcome sentence."));

        let record = db
            .get_summary_and_title(555)
            .await
            .expect("record must exist");
        assert_eq!(record.summary, "Initial brief");
        assert_eq!(
            record.rewrite.as_deref(),
            Some(
                "Single rewrite lead sentence. High impact outcome sentence.\n\n• Context point 1."
            )
        );
    }

    #[test]
    fn test_canonical_page_key_consistency() {
        let key1 = compute_page_key("unread", &[1, 2, 3]);
        let key2 = compute_page_key("unread", &[1, 2, 3]);
        assert_eq!(key1, key2, "Canonical page key must be strictly consistent");

        let empty_key = compute_page_key("unread", &[]);
        assert!(
            empty_key.starts_with("unread:"),
            "Empty slice key must start with prefix"
        );
        assert_eq!(
            empty_key.len(),
            "unread:".len() + 16,
            "Empty slice key must have 16-char hash"
        );
    }

    #[test]
    fn test_compute_page_key() {
        let key1 = compute_page_key("unread", &[101, 102, 103]);
        let key2 = compute_page_key("unread", &[101, 102, 103]);
        assert_eq!(key1, key2, "same inputs produce identical key");
        assert!(
            key1.starts_with("unread:"),
            "must prefix with status: {}",
            key1
        );
        assert_eq!(
            key1.len(),
            "unread:".len() + 16,
            "16 hex chars after prefix"
        );

        let key_diff_order = compute_page_key("unread", &[103, 102, 101]);
        assert_ne!(
            key1, key_diff_order,
            "different order produces different key"
        );

        let key_diff_status = compute_page_key("all", &[101, 102, 103]);
        assert_ne!(
            key1, key_diff_status,
            "different status produces different key"
        );
    }

    #[tokio::test]
    async fn test_ai_service_analyze_entry_reuses_existing_db_record() {
        let db = Arc::new(AiDb::new_mock());
        let extractor = ArticleExtractor::new();

        // Runner points to an invalid binary so if it is called, it would fail
        let llm = LlmCliRunner::new(
            "/bin/nonexistent_llm_binary_that_should_not_be_called".to_string(),
            "test-model".to_string(),
            vec![],
        );

        let ai = AiService {
            db: db.clone(),
            extractor,
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        };

        // Seed DB with existing analysis
        db.save_summary_and_title_with_analysis(
            777,
            Some("Suggested Title 777"),
            "Summary 777",
            Some("Rewrite 777"),
            Some("Existing DB Impact"),
            Some("Existing DB Fallacies"),
            "test-model",
        )
        .await
        .unwrap();

        let res = ai
            .analyze_entry_impact_and_fallacies(777, "Title 777", "Content 777")
            .await;

        assert!(
            res.is_ok(),
            "Should return Ok from DB cache without calling LLM"
        );
        let (impact, fallacies) = res.unwrap();
        assert_eq!(impact, "Existing DB Impact");
        assert_eq!(fallacies, "Existing DB Fallacies");
    }
}
