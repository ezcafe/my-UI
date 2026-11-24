use crate::ai::AiService;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PrefetchCandidate {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub teaser: String,
    pub feed_title: String,
    pub published_at: String,
}

impl From<(i64, String, String, String)> for PrefetchCandidate {
    fn from((id, title, url, teaser): (i64, String, String, String)) -> Self {
        Self {
            id,
            title,
            url,
            teaser,
            feed_title: "General".to_string(),
            published_at: String::new(),
        }
    }
}

#[derive(Clone)]
pub struct AiPrefetchService {
    ai_service: Arc<AiService>,
}

impl AiPrefetchService {
    pub fn new(ai_service: Arc<AiService>) -> Self {
        Self { ai_service }
    }

    /// Spawns background worker to prefetch summaries for uncached entries.
    pub fn spawn_prefetch_job<T: Into<PrefetchCandidate> + Send + 'static>(
        &self,
        status: String,
        page_entry_ids: Vec<i64>,
        candidate_entries: Vec<T>,
    ) {
        let ai = self.ai_service.clone();
        tokio::spawn(async move {
            Self::run_prefetch(ai, status, page_entry_ids, candidate_entries).await;
        });
    }

    pub async fn run_prefetch<T: Into<PrefetchCandidate>>(
        ai: Arc<AiService>,
        status: String,
        page_entry_ids: Vec<i64>,
        candidate_entries: Vec<T>,
    ) {
        let candidate_entries: Vec<PrefetchCandidate> =
            candidate_entries.into_iter().map(Into::into).collect();
        if candidate_entries.is_empty() {
            return;
        }

        // 1. Filter out already cached IDs using batched unprocessed check
        let ids: Vec<i64> = candidate_entries.iter().map(|c| c.id).collect();
        let unprocessed_ids = ai.db.get_unprocessed_entry_ids(&ids).await;
        let unprocessed_set: std::collections::HashSet<i64> = unprocessed_ids.into_iter().collect();

        let uncached: Vec<PrefetchCandidate> = candidate_entries
            .into_iter()
            .filter(|c| unprocessed_set.contains(&c.id))
            .collect();

        // 2. Prefetch threshold: trigger when uncached items >= 1
        const PREFETCH_THRESHOLD: usize = 1;
        if uncached.len() < PREFETCH_THRESHOLD {
            return;
        }

        let batch_targets: Vec<_> = uncached.into_iter().take(20).collect();
        let batch_key = if !page_entry_ids.is_empty() {
            crate::ai::compute_page_key(&status, &page_entry_ids)
        } else {
            let target_ids: Vec<i64> = batch_targets.iter().map(|c| c.id).collect();
            crate::ai::compute_page_key(&status, &target_ids)
        };

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
            key: batch_key.clone(),
        };

        // 4. Single-call batch prefetch: process all candidate page entries in exactly 1 LLM call
        // Extract article contents concurrently to minimize latency before batch LLM call
        let mut join_set = tokio::task::JoinSet::new();
        for target in &batch_targets {
            let extractor = ai.extractor.clone();
            let id = target.id;
            let title = target.title.clone();
            let url = target.url.clone();
            let teaser = target.teaser.clone();
            let feed_title = target.feed_title.clone();
            let published_at = target.published_at.clone();
            join_set.spawn(async move {
                let content = extractor.extract_content(&url, &title, &teaser).await;
                (id, title, feed_title, published_at, content)
            });
        }

        let mut extracted_contents = Vec::new();
        while let Some(res) = join_set.join_next().await {
            if let Ok(item) = res {
                extracted_contents.push(item);
            }
        }

        let articles_for_llm: Vec<crate::ai::llm::BatchArticleInput> = extracted_contents
            .iter()
            .map(
                |(id, title, source, published_at, content)| crate::ai::llm::BatchArticleInput {
                    id: *id,
                    title: title.as_str(),
                    source: source.as_str(),
                    published_at: published_at.as_str(),
                    content_or_brief: content.as_str(),
                    is_cached: false,
                },
            )
            .collect();

        let (batch_res, model) = ai.llm.summarize_batch_structured(&articles_for_llm).await;

        // Save page executive brief if top is provided (both with page_key and legacy)
        if !batch_res.top.is_empty() {
            let _ = ai
                .db
                .save_page_brief_with_key(&batch_key, &batch_res.top)
                .await;
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
                .save_summary_and_title_with_rewrite(
                    item.id,
                    suggested_title,
                    &item.brief,
                    item.rewrite.as_deref(),
                    &model,
                )
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
        let db = Arc::new(AiDb::new_mock());
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
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        });

        // Pass 15 candidate entries
        let candidates: Vec<(i64, String, String, String)> = (1..=15)
            .map(|i| {
                (
                    i,
                    format!("Title {i}"),
                    format!("http://example.com/{i}"),
                    format!("Teaser {i}"),
                )
            })
            .collect();

        AiPrefetchService::run_prefetch(ai.clone(), "unread".to_string(), vec![], candidates).await;

        let calls = std::fs::read_to_string(&count_path).unwrap();
        let call_count = calls.lines().count();
        // Must be exactly 1 call (no chunking into 12)
        assert_eq!(
            call_count, 1,
            "Expected exactly 1 LLM call for all candidates, but got {}",
            call_count
        );

        let page_brief = db.get_latest_page_brief().await;
        assert_eq!(page_brief.as_deref(), Some("Daily Brief"));
    }

    #[tokio::test]
    async fn test_prefetch_and_executive_brief_share_page_key() {
        let db = Arc::new(AiDb::new_mock());
        let extractor = ArticleExtractor::new();

        let count_file = tempfile::NamedTempFile::new().unwrap();
        let count_path = count_file.path().to_str().unwrap().to_string();

        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        let script_content = format!(
            "#!/bin/sh\nprintf 'call\\n' >> \"{}\"\nprintf '%s' '{{\"top\":\"Canonical Key Brief\",\"items\":[]}}'\n",
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
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        });

        let _prefetch_service = AiPrefetchService::new(ai.clone());
        let page_ids = vec![601, 602, 603];
        let candidates = vec![(
            601,
            "T601".to_string(),
            "http://example.com/601".to_string(),
            "Teaser 601".to_string(),
        )];

        let expected_canonical_key = crate::ai::compute_page_key("unread", &page_ids);

        // Run prefetch using canonical page status and page entry ids
        AiPrefetchService::run_prefetch(
            ai.clone(),
            "unread".to_string(),
            page_ids.clone(),
            candidates,
        )
        .await;

        let keyed_brief = db.get_page_brief_by_key(&expected_canonical_key).await;
        assert_eq!(
            keyed_brief.as_deref(),
            Some("Canonical Key Brief"),
            "Prefetch must store brief using canonical page_key"
        );
    }

    #[tokio::test]
    async fn test_prefetch_saves_brief_with_page_key() {
        let db = Arc::new(AiDb::new_mock());
        let extractor = ArticleExtractor::new();

        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        let script_content =
            "#!/bin/sh\nprintf '%s' '{\"top\":\"Keyed Prefetch Brief\",\"items\":[]}'\n";
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
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        });

        let candidates = vec![(
            501,
            "T1".to_string(),
            "http://example.com/1".to_string(),
            "Teaser 1".to_string(),
        )];
        let expected_key = crate::ai::compute_page_key("unread", &[501]);

        AiPrefetchService::run_prefetch(ai.clone(), "unread".to_string(), vec![501], candidates)
            .await;

        let keyed_brief = db.get_page_brief_by_key(&expected_key).await;
        assert_eq!(keyed_brief.as_deref(), Some("Keyed Prefetch Brief"));
    }

    #[tokio::test]
    async fn test_prefetch_triggers_with_single_uncached_item() {
        let db = Arc::new(AiDb::new_mock());
        let extractor = ArticleExtractor::new();

        let count_file = tempfile::NamedTempFile::new().unwrap();
        let count_path = count_file.path().to_str().unwrap().to_string();

        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        let script_content = format!(
            "#!/bin/sh\nprintf 'call\\n' >> \"{}\"\nprintf '%s' '{{\"top\":\"Single Item Brief\",\"items\":[]}}'\n",
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
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        });

        // Exactly 1 uncached candidate item
        let single_candidate = vec![(
            999,
            "Title 999".to_string(),
            "http://example.com/999".to_string(),
            "Teaser 999".to_string(),
        )];

        AiPrefetchService::run_prefetch(ai.clone(), "unread".to_string(), vec![], single_candidate)
            .await;

        let calls = std::fs::read_to_string(&count_path).unwrap_or_default();
        assert_eq!(
            calls.lines().count(),
            1,
            "Lowered threshold (1) must trigger prefetch for 1 uncached item"
        );
    }

    #[tokio::test]
    async fn test_prefetch_threshold_and_duplicate_avoidance() {
        let db = Arc::new(AiDb::new_mock());
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
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        });

        // Case 1: 0 candidates -> should NOT trigger LLM call
        let empty_candidates: Vec<(i64, String, String, String)> = vec![];

        AiPrefetchService::run_prefetch(ai.clone(), "unread".to_string(), vec![], empty_candidates)
            .await;
        let calls = std::fs::read_to_string(&count_path).unwrap_or_default();
        assert_eq!(
            calls.lines().count(),
            0,
            "Empty candidates must not trigger prefetch"
        );

        // Case 2: Pre-populate all items so 0 remain uncached -> should NOT trigger
        for i in 101..=104 {
            db.save_summary(i, "Summary", "gpt-4o-mini").await.unwrap();
        }
        let all_cached_candidates: Vec<(i64, String, String, String)> = (101..=104)
            .map(|i| {
                (
                    i,
                    format!("Title {i}"),
                    format!("http://example.com/{i}"),
                    format!("Teaser {i}"),
                )
            })
            .collect();

        AiPrefetchService::run_prefetch(
            ai.clone(),
            "unread".to_string(),
            vec![],
            all_cached_candidates,
        )
        .await;
        let calls = std::fs::read_to_string(&count_path).unwrap_or_default();
        assert_eq!(
            calls.lines().count(),
            0,
            "All cached items must not trigger prefetch"
        );

        // Case 3: 1 uncached item (>= 1 threshold) -> triggers LLM call
        let threshold_candidates: Vec<(i64, String, String, String)> = vec![(
            201,
            "Title 201".to_string(),
            "http://example.com/201".to_string(),
            "Teaser 201".to_string(),
        )];

        AiPrefetchService::run_prefetch(
            ai.clone(),
            "unread".to_string(),
            vec![],
            threshold_candidates,
        )
        .await;
        let calls = std::fs::read_to_string(&count_path).unwrap();
        assert_eq!(
            calls.lines().count(),
            1,
            "1 uncached item must trigger prefetch call exactly once"
        );
    }

    #[tokio::test]
    async fn test_prefetch_service_persists_rewrites() {
        let db = Arc::new(AiDb::new_mock());
        let extractor = ArticleExtractor::new();

        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        let script_content = "#!/bin/sh\nprintf '%s' '{\"top\":\"Brief Top\",\"items\":[{\"i\":1,\"t\":\"T1\",\"b\":\"B1\",\"r\":\"Inverted pyramid lead.\\n\\n• Supporting detail.\"}]}'\n";
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
            llm: llm.into(),
            in_flight: Arc::new(tokio::sync::Mutex::new(std::collections::HashSet::new())),
        });

        let prefetch_service = AiPrefetchService::new(ai.clone());
        let candidates: Vec<PrefetchCandidate> = vec![PrefetchCandidate {
            id: 1,
            title: "Test Title".to_string(),
            url: "http://example.com/1".to_string(),
            teaser: "Teaser content".to_string(),
            feed_title: "Tech News".to_string(),
            published_at: "2026-11-23T12:00:00Z".to_string(),
        }];

        prefetch_service.spawn_prefetch_job("unread".to_string(), vec![1], candidates);

        // Await background completion
        let mut persisted = false;
        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            if let Some(record) = db.get_summary_and_title(1).await {
                if record.rewrite.is_some() {
                    persisted = true;
                    assert_eq!(
                        record.rewrite.as_deref(),
                        Some("Inverted pyramid lead.\n\n• Supporting detail.")
                    );
                    assert_eq!(record.summary, "B1");
                    assert_eq!(record.suggested_title.as_deref(), Some("T1"));
                    break;
                }
            }
        }
        assert!(
            persisted,
            "AiPrefetchService must persist rewrite into AiDb"
        );
    }
}
