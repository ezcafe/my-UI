use crate::ai::AiService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AiPrefetchService {
    ai_service: Arc<AiService>,
}

impl AiPrefetchService {
    pub fn new(ai_service: Arc<AiService>) -> Self {
        Self { ai_service }
    }

    /// Spawns background worker to prefetch summaries for uncached entries.
    /// candidate_entries is a list of (id, title, url, teaser) tuples.
    pub fn spawn_prefetch_job(&self, candidate_entries: Vec<(i64, String, String, String)>) {
        let ai = self.ai_service.clone();
        tokio::spawn(async move {
            Self::run_prefetch(ai, candidate_entries).await;
        });
    }

    pub async fn run_prefetch(
        ai: Arc<AiService>,
        candidate_entries: Vec<(i64, String, String, String)>,
    ) {
        if candidate_entries.is_empty() {
            return;
        }

        // 1. Filter out already cached IDs using batched unprocessed check
        let ids: Vec<i64> = candidate_entries.iter().map(|(id, ..)| *id).collect();
        let unprocessed_ids = ai.db.get_unprocessed_entry_ids(&ids).await;
        let unprocessed_set: std::collections::HashSet<i64> = unprocessed_ids.into_iter().collect();

        let uncached: Vec<(i64, String, String, String)> = candidate_entries
            .into_iter()
            .filter(|(id, ..)| unprocessed_set.contains(id))
            .collect();

        // 2. Prefetch threshold: only trigger when uncached items >= 5
        const PREFETCH_THRESHOLD: usize = 5;
        if uncached.len() < PREFETCH_THRESHOLD {
            return;
        }

        let batch_targets: Vec<_> = uncached.into_iter().take(20).collect();
        let target_ids: Vec<i64> = batch_targets.iter().map(|(id, ..)| *id).collect();
        let batch_key = crate::ai::compute_page_key("prefetch", &target_ids);

        // 3. Concurrency guard: avoid duplicate in-flight batch runs
        {
            let mut in_flight = ai.in_flight.lock().await;
            if in_flight.contains(&batch_key) {
                return;
            }
            in_flight.insert(batch_key.clone());
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
            in_flight: ai.in_flight.clone(),
            key: batch_key,
        };

        // 4. Single-call batch prefetch: process all candidate page entries in exactly 1 LLM call
        // Extract article contents concurrently to minimize latency before batch LLM call
        let mut join_set = tokio::task::JoinSet::new();
        for (id, title, url, teaser) in &batch_targets {
            let extractor = ai.extractor.clone();
            let id = *id;
            let title = title.clone();
            let url = url.clone();
            let teaser = teaser.clone();
            join_set.spawn(async move {
                let content = extractor.extract_content(&url, &title, &teaser).await;
                (id, title, content)
            });
        }

        let mut extracted_contents = Vec::new();
        while let Some(res) = join_set.join_next().await {
            if let Ok(item) = res {
                extracted_contents.push(item);
            }
        }

        let mut articles_for_llm = Vec::new();
        for (id, title, content) in &extracted_contents {
            articles_for_llm.push((*id, title.as_str(), content.as_str()));
        }

        let (batch_res, model) = ai.llm.summarize_batch(&articles_for_llm).await;

        // Save page executive brief if top is provided
        if !batch_res.top.is_empty() {
            let _ = ai.db.save_page_brief(&batch_res.top).await;
        }

        // Save individual items
        for item in batch_res.items {
            let suggested_title = if item.suggested_title.is_empty() {
                None
            } else {
                Some(item.suggested_title.as_str())
            };
            let _ = ai
                .db
                .save_summary_and_title(item.id, suggested_title, &item.brief, &model)
                .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{AiDb, ArticleExtractor, LlmCliRunner};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    async fn test_page_batch_processing() {
        let db_file = tempfile::NamedTempFile::new().unwrap();
        let db = Arc::new(AiDb::new(db_file.path().to_str().unwrap()).unwrap());
        let extractor = ArticleExtractor::new();

        let count_file = tempfile::NamedTempFile::new().unwrap();
        let count_path = count_file.path().to_str().unwrap().to_string();

        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        let script_content = format!(
            "#!/bin/sh\nprintf 'call\\n' >> \"{}\"\nprintf '%s' '{{\"top\":\"Daily Brief\",\"items\":[{{\"i\":1,\"t\":\"T1\",\"b\":\"B1\"}}]}}'\n",
            count_path
        );
        script_file.write_all(script_content.as_bytes()).unwrap();
        let mut perms = std::fs::metadata(script_file.path()).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(script_file.path(), perms).unwrap();

        let llm = LlmCliRunner::new(
            script_file.path().to_str().unwrap().to_string(),
            "test-model".to_string(),
            vec![],
        );

        let ai = Arc::new(AiService {
            db: db.clone(),
            extractor,
            llm,
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        });

        // Pass 15 candidate entries
        let candidates: Vec<(i64, String, String, String)> = (1..=15)
            .map(|i| (i, format!("Title {i}"), format!("http://example.com/{i}"), format!("Teaser {i}")))
            .collect();

        AiPrefetchService::run_prefetch(ai.clone(), candidates).await;

        let calls = std::fs::read_to_string(&count_path).unwrap();
        let call_count = calls.lines().count();
        // Must be exactly 1 call (no chunking into 12)
        assert_eq!(call_count, 1, "Expected exactly 1 LLM call for all candidates, but got {}", call_count);

        let page_brief = db.get_latest_page_brief().await;
        assert_eq!(page_brief.as_deref(), Some("Daily Brief"));
    }

    #[tokio::test]
    async fn test_prefetch_threshold_and_duplicate_avoidance() {
        let db_file = tempfile::NamedTempFile::new().unwrap();
        let db = Arc::new(AiDb::new(db_file.path().to_str().unwrap()).unwrap());
        let extractor = ArticleExtractor::new();

        let count_file = tempfile::NamedTempFile::new().unwrap();
        let count_path = count_file.path().to_str().unwrap().to_string();

        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        let script_content = format!(
            "#!/bin/sh\nprintf 'call\\n' >> \"{}\"\nprintf '%s' '{{\"top\":\"Daily Brief\",\"items\":[]}}'\n",
            count_path
        );
        script_file.write_all(script_content.as_bytes()).unwrap();
        let mut perms = std::fs::metadata(script_file.path()).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(script_file.path(), perms).unwrap();

        let llm = LlmCliRunner::new(
            script_file.path().to_str().unwrap().to_string(),
            "test-model".to_string(),
            vec![],
        );

        let ai = Arc::new(AiService {
            db: db.clone(),
            extractor,
            llm,
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        });

        // Case 1: Uncached count < 5 (e.g. 3 entries) -> should NOT trigger LLM call
        let below_threshold_candidates: Vec<(i64, String, String, String)> = (1..=3)
            .map(|i| (i, format!("Title {i}"), format!("http://example.com/{i}"), format!("Teaser {i}")))
            .collect();

        AiPrefetchService::run_prefetch(ai.clone(), below_threshold_candidates).await;
        let calls = std::fs::read_to_string(&count_path).unwrap_or_default();
        assert_eq!(calls.lines().count(), 0, "Uncached count below 5 must not trigger prefetch");

        // Case 2: Pre-populate 4 out of 6 items so only 2 remain uncached -> should NOT trigger
        for i in 101..=104 {
            db.save_summary(i, "Summary", "gpt-4o-mini").await.unwrap();
        }
        let six_candidates: Vec<(i64, String, String, String)> = (101..=106)
            .map(|i| (i, format!("Title {i}"), format!("http://example.com/{i}"), format!("Teaser {i}")))
            .collect();

        AiPrefetchService::run_prefetch(ai.clone(), six_candidates).await;
        let calls = std::fs::read_to_string(&count_path).unwrap_or_default();
        assert_eq!(calls.lines().count(), 0, "Only 2 uncached items remaining must not trigger prefetch");

        // Case 3: 5 uncached items (>= 5 threshold) -> triggers LLM call
        let threshold_candidates: Vec<(i64, String, String, String)> = (201..=205)
            .map(|i| (i, format!("Title {i}"), format!("http://example.com/{i}"), format!("Teaser {i}")))
            .collect();

        AiPrefetchService::run_prefetch(ai.clone(), threshold_candidates).await;
        let calls = std::fs::read_to_string(&count_path).unwrap();
        assert_eq!(calls.lines().count(), 1, "5 uncached items must trigger prefetch call exactly once");
    }
}
