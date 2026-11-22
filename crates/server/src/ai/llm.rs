use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Semaphore;

static TOP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""top"\s*:\s*"([^"\\]*(?:\\.[^"\\]*)*)""#).expect("valid top regex")
});

static ITEM_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"\{[^{}]*?"i"\s*:\s*(\d+)[^{}]*?"t"\s*:\s*"([^"\\]*(?:\\.[^"\\]*)*)"[^{}]*?"b"\s*:\s*"([^"\\]*(?:\\.[^"\\]*)*)"[^{}]*?\}"#,
    )
    .expect("valid item regex")
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BatchAiItem {
    #[serde(rename = "i")]
    pub id: i64,
    #[serde(rename = "t")]
    pub suggested_title: String,
    #[serde(rename = "b")]
    pub brief: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BatchAiResponse {
    #[serde(default)]
    pub top: String,
    #[serde(default)]
    pub items: Vec<BatchAiItem>,
}

#[derive(Clone)]
pub struct LlmCliRunner {
    pub cli_cmd: String,
    pub model: String,
    pub extra_args: Vec<String>,
    pub semaphore: Arc<Semaphore>,
}

impl LlmCliRunner {
    pub fn new(cli_cmd: String, model: String, extra_args: Vec<String>) -> Self {
        Self {
            cli_cmd,
            model,
            extra_args,
            semaphore: Arc::new(Semaphore::new(2)),
        }
    }

    pub fn format_prompt(title: &str, content: &str) -> String {
        format!(
            "Summarize what changed or why this news matters in 1 to 2 concise sentences (strictly under 35 words). Do not include introductory filler.\n\nArticle Title: {}\nArticle Content:\n{}",
            title.trim(),
            content.trim()
        )
    }

    pub fn format_batch_prompt(articles: &[(i64, &str, &str)]) -> String {
        let mut prompt = String::from(
            "You are an executive news editor. For the articles below, provide:\n\
             1. \"top\": A 1-2 sentence executive brief (<35 words) synthesizing the most important themes.\n\
             2. \"items\": For each article id, a punchy \"t\" (suggested title, <=10 words) and \"b\" (executive brief, 1-2 sentences, <35 words, what changed/why it matters).\n\n\
             CRITICAL MULTILINGUAL RULE:\n\
             For each article and for the overall brief, write your response in the EXACT SAME LANGUAGE as the source text.\n\
             If an article is in Vietnamese, its suggested title and brief MUST be in Vietnamese.\n\
             If an article is in English, its suggested title and brief MUST be in English.\n\
             Do NOT translate articles into a different language.\n\n\
             Output raw JSON only using this compact schema:\n\
             {\"top\":\"...\",\"items\":[{\"i\":101,\"t\":\"...\",\"b\":\"...\"}]}\n\n\
             Articles:\n",
        );
        for (id, title, content) in articles {
            prompt.push_str(&format!(
                "[ID {}] Title: {}\nContent:\n{}\n\n",
                id,
                title.trim(),
                content.trim()
            ));
        }
        prompt
    }

    pub fn build_args(&self, prompt: &str) -> Vec<String> {
        let mut args = vec!["-m".to_string(), self.model.clone()];
        args.extend(self.extra_args.clone());
        args.push(prompt.to_string());
        args
    }

    pub fn format_dispatch_log(model: &str, item_count: usize, prompt: &str) -> String {
        format!(
            "[AI_DISPATCH] model=\"{}\" item_count={} prompt_preview={:?}",
            model,
            item_count,
            prompt.chars().take(200).collect::<String>()
        )
    }

    pub fn format_return_log(elapsed_ms: u128, raw_output: &str) -> String {
        format!(
            "[AI_RETURN] elapsed_ms={} raw_output={:?}",
            elapsed_ms, raw_output
        )
    }

    pub fn format_warn_fallback(reason: &str, fallback_strategy: &str, snippet: &str) -> String {
        format!(
            "[AI_WARN] reason=\"{}\" fallback_strategy=\"{}\" raw_snippet={:?}",
            reason,
            fallback_strategy,
            snippet.chars().take(150).collect::<String>()
        )
    }

    pub fn format_error_timeout(timeout_secs: u64, elapsed_ms: u128) -> String {
        format!(
            "[AI_ERROR] error=\"cli_timeout\" timeout_secs={} elapsed_ms={}",
            timeout_secs, elapsed_ms
        )
    }

    pub fn format_error_process_failed(exit_code: Option<i32>, stderr: &str) -> String {
        format!(
            "[AI_ERROR] error=\"process_failed\" exit_code={:?} stderr={:?}",
            exit_code,
            stderr.chars().take(200).collect::<String>()
        )
    }

    pub fn format_error_empty_output(elapsed_ms: u128) -> String {
        format!(
            "[AI_ERROR] error=\"empty_output\" elapsed_ms={}",
            elapsed_ms
        )
    }

    pub fn parse_batch_response(raw_output: &str) -> BatchAiResponse {
        let trimmed = raw_output.trim();

        // 1. Try direct serde_json parse
        if let Ok(res) = serde_json::from_str::<BatchAiResponse>(trimmed) {
            tracing::info!(
                target: "ai::batch",
                "[AI_SUCCESS] batch_size={} top_brief={}",
                res.items.len(),
                !res.top.is_empty()
            );
            return res;
        }

        // 2. Try extracting from markdown code block ```json ... ``` or ``` ... ```
        if let Some(start) = trimmed.find("```") {
            let after_start = &trimmed[start + 3..];
            let code_content = if let Some(first_newline) = after_start.find('\n') {
                &after_start[first_newline + 1..]
            } else {
                after_start
            };
            if let Some(end) = code_content.rfind("```") {
                let candidate = code_content[..end].trim();
                if let Ok(res) = serde_json::from_str::<BatchAiResponse>(candidate) {
                    tracing::info!(
                        target: "ai::batch",
                        "[AI_SUCCESS] (fenced) batch_size={} top_brief={}",
                        res.items.len(),
                        !res.top.is_empty()
                    );
                    return res;
                }
            }
        }

        // 3. Fallback regex extraction
        let snippet: String = trimmed.chars().take(150).collect();
        tracing::warn!(
            target: "ai::batch",
            "{}",
            Self::format_warn_fallback("json_parse_fallback", "regex", &snippet)
        );

        let top = TOP_REGEX
            .captures(trimmed)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().replace(r#"\""#, "\""))
            .unwrap_or_default();

        let mut items = Vec::new();
        for cap in ITEM_REGEX.captures_iter(trimmed) {
            if let (Some(id_str), Some(title), Some(brief)) = (cap.get(1), cap.get(2), cap.get(3)) {
                if let Ok(id) = id_str.as_str().parse::<i64>() {
                    items.push(BatchAiItem {
                        id,
                        suggested_title: title.as_str().replace(r#"\""#, "\""),
                        brief: brief.as_str().replace(r#"\""#, "\""),
                    });
                }
            }
        }

        if !items.is_empty() || !top.is_empty() {
            return BatchAiResponse { top, items };
        }

        tracing::error!(
            target: "ai::batch",
            "[AI_ERROR] error=\"parse_failed\" action=\"empty_fallback\""
        );
        BatchAiResponse::default()
    }

    pub async fn summarize_batch(
        &self,
        articles: &[(i64, &str, &str)],
    ) -> (BatchAiResponse, String) {
        if articles.is_empty() {
            return (BatchAiResponse::default(), self.model.clone());
        }

        let _permit = self.semaphore.acquire().await.ok();
        let prompt = Self::format_batch_prompt(articles);
        let args = self.build_args(&prompt);

        let dispatch_msg = Self::format_dispatch_log(&self.model, articles.len(), &prompt);
        println!("{}", dispatch_msg);
        tracing::info!(
            target: "ai::batch",
            "{}",
            dispatch_msg
        );

        let start = std::time::Instant::now();
        const TIMEOUT_SECS: u64 = 90;
        let result = tokio::time::timeout(Duration::from_secs(TIMEOUT_SECS), async {
            let mut cmd = Command::new(&self.cli_cmd);
            cmd.args(&args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);

            cmd.output().await
        })
        .await;

        match result {
            Ok(Ok(output)) => {
                let elapsed = start.elapsed().as_millis();
                if !output.status.success() {
                    let stderr_preview = String::from_utf8_lossy(&output.stderr);
                    let err_log = Self::format_error_process_failed(output.status.code(), &stderr_preview);
                    eprintln!("{}", err_log);
                    tracing::error!(
                        target: "ai::batch",
                        "{}",
                        err_log
                    );
                    Self::fallback_extractive(articles)
                } else {
                    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if text.is_empty() {
                        let err_log = Self::format_error_empty_output(elapsed);
                        eprintln!("{}", err_log);
                        tracing::error!(
                            target: "ai::batch",
                            "{}",
                            err_log
                        );
                        Self::fallback_extractive(articles)
                    } else {
                        let ret_log = Self::format_return_log(elapsed, &text);
                        println!("{}", ret_log);
                        tracing::info!(
                            target: "ai::batch",
                            "{}",
                            ret_log
                        );
                        let parsed = Self::parse_batch_response(&text);
                        (parsed, self.model.clone())
                    }
                }
            }
            Ok(Err(e)) => {
                let err_str = e.to_string();
                let err_log = Self::format_error_process_failed(None, &err_str);
                eprintln!("{}", err_log);
                tracing::error!(
                    target: "ai::batch",
                    "{}",
                    err_log
                );
                Self::fallback_extractive(articles)
            }
            Err(_) => {
                let elapsed = start.elapsed().as_millis();
                let err_log = Self::format_error_timeout(TIMEOUT_SECS, elapsed);
                eprintln!("{}", err_log);
                tracing::error!(
                    target: "ai::batch",
                    "{}",
                    err_log
                );
                Self::fallback_extractive(articles)
            }
        }
    }

    fn fallback_extractive(articles: &[(i64, &str, &str)]) -> (BatchAiResponse, String) {
        tracing::warn!(
            target: "ai::batch",
            "[AI_WARN] reason=\"cli_failure\" fallback_strategy=\"extractive\""
        );
        let mut items = Vec::new();
        for (id, _title, content) in articles {
            let fallback = miniflux_client::summary::extract_summary(content, 2);
            let fallback_text = if fallback.is_empty() {
                content.trim().to_string()
            } else {
                fallback
            };
            items.push(BatchAiItem {
                id: *id,
                suggested_title: String::new(),
                brief: fallback_text,
            });
        }
        (
            BatchAiResponse {
                top: String::new(),
                items,
            },
            "extractive-fallback".to_string(),
        )
    }

    pub async fn summarize(&self, title: &str, content: &str) -> (String, String) {
        let _permit = self.semaphore.acquire().await.ok();

        let prompt = Self::format_prompt(title, content);
        let args = self.build_args(&prompt);

        let dispatch_msg = format!(
            "[AI_DISPATCH_SINGLE] model={} title=\"{}\" prompt_preview=\"{}\"",
            self.model,
            title.trim(),
            prompt.chars().take(150).collect::<String>().replace('\n', " ")
        );
        println!("{}", dispatch_msg);
        tracing::info!(target: "ai::single", "{}", dispatch_msg);

        let start = std::time::Instant::now();
        const TIMEOUT_SECS: u64 = 30;
        let result = tokio::time::timeout(Duration::from_secs(TIMEOUT_SECS), async {
            let mut cmd = Command::new(&self.cli_cmd);
            cmd.args(&args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);

            cmd.output().await
        })
        .await;

        match result {
            Ok(Ok(output)) => {
                let elapsed = start.elapsed().as_millis();
                if !output.status.success() {
                    let stderr_preview = String::from_utf8_lossy(&output.stderr);
                    let err_log = format!(
                        "[AI_ERROR_SINGLE] status={:?} stderr=\"{}\"",
                        output.status.code(),
                        stderr_preview.trim().replace('\n', " ")
                    );
                    eprintln!("{}", err_log);
                    tracing::error!(target: "ai::single", "{}", err_log);
                    let fallback = miniflux_client::summary::extract_summary(content, 2);
                    let fallback_text = if fallback.is_empty() { content.trim().to_string() } else { fallback };
                    (fallback_text, "extractive-fallback".to_string())
                } else {
                    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if text.is_empty() {
                        let err_log = format!("[AI_ERROR_SINGLE] empty_output elapsed_ms={}", elapsed);
                        eprintln!("{}", err_log);
                        tracing::error!(target: "ai::single", "{}", err_log);
                        let fallback = miniflux_client::summary::extract_summary(content, 2);
                        let fallback_text = if fallback.is_empty() { content.trim().to_string() } else { fallback };
                        (fallback_text, "extractive-fallback".to_string())
                    } else {
                        let return_msg = format!(
                            "[AI_RETURN_SINGLE] elapsed_ms={} output=\"{}\"",
                            elapsed,
                            text.chars().take(200).collect::<String>().replace('\n', " ")
                        );
                        println!("{}", return_msg);
                        tracing::info!(target: "ai::single", "{}", return_msg);
                        (text, self.model.clone())
                    }
                }
            }
            Ok(Err(e)) => {
                let err_log = format!("[AI_ERROR_SINGLE] process_error=\"{}\"", e);
                eprintln!("{}", err_log);
                tracing::error!(target: "ai::single", "{}", err_log);
                let fallback = miniflux_client::summary::extract_summary(content, 2);
                let fallback_text = if fallback.is_empty() { content.trim().to_string() } else { fallback };
                (fallback_text, "extractive-fallback".to_string())
            }
            Err(_) => {
                let elapsed = start.elapsed().as_millis();
                let err_log = format!("[AI_ERROR_SINGLE] timeout_after_{}s elapsed_ms={}", TIMEOUT_SECS, elapsed);
                eprintln!("{}", err_log);
                tracing::error!(target: "ai::single", "{}", err_log);
                let fallback = miniflux_client::summary::extract_summary(content, 2);
                let fallback_text = if fallback.is_empty() { content.trim().to_string() } else { fallback };
                (fallback_text, "extractive-fallback".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_prompt_formatting_and_args() {
        let runner = LlmCliRunner::new(
            "llm".to_string(),
            "gemini-2.5-flash".to_string(),
            vec!["--thinking-budget".to_string(), "0".to_string()],
        );

        let articles = [
            (101, "Article 1", "Content of first article"),
            (102, "Article 2", "Content of second article"),
        ];
        let prompt = LlmCliRunner::format_batch_prompt(&articles);
        assert!(prompt.contains(r#"{"top":"...","items":[{"i":101,"t":"...","b":"..."}]"#));
        assert!(prompt.contains("[ID 101] Title: Article 1"));
        assert!(prompt.contains("[ID 102] Title: Article 2"));
        assert!(prompt.contains("Content of first article"));
        assert!(prompt.contains("Content of second article"));

        let args = runner.build_args(&prompt);
        assert!(args.contains(&"--thinking-budget".to_string()));
        assert!(args.contains(&"0".to_string()));
    }

    #[test]
    fn test_multilingual_batch_prompt_and_robust_parsing() {
        let articles = [
            (101, "Tin tức công nghệ", "Nội dung bài viết tiếng Việt về trí tuệ nhân tạo."),
            (102, "Global Markets", "Stock indexes reached all-time highs today."),
        ];
        let prompt = LlmCliRunner::format_batch_prompt(&articles);

        // Verify critical multilingual instructions are present in prompt
        assert!(prompt.contains("CRITICAL MULTILINGUAL RULE:"));
        assert!(prompt.contains("EXACT SAME LANGUAGE as the source text"));
        assert!(prompt.contains("If an article is in Vietnamese, its suggested title and brief MUST be in Vietnamese."));
        assert!(prompt.contains("Do NOT translate articles into a different language."));

        // Verify parsing multilingual response with markdown fences and conversational preamble
        let raw_multilingual = r#"
Sure! Here is the JSON summary:
```json
{
  "top": "Tổng hợp tin tức công nghệ và thị trường tài chính quốc tế.",
  "items": [
    { "i": 101, "t": "Đột phá AI mới", "b": "Công nghệ AI ghi nhận bước tiến quan trọng." },
    { "i": 102, "t": "Market Highs", "b": "Global stock indices rally to record levels." }
  ]
}
```
Have a great day!
        "#;
        let parsed = LlmCliRunner::parse_batch_response(raw_multilingual);
        assert_eq!(parsed.top, "Tổng hợp tin tức công nghệ và thị trường tài chính quốc tế.");
        assert_eq!(parsed.items.len(), 2);
        assert_eq!(parsed.items[0].id, 101);
        assert_eq!(parsed.items[0].suggested_title, "Đột phá AI mới");
        assert_eq!(parsed.items[0].brief, "Công nghệ AI ghi nhận bước tiến quan trọng.");
        assert_eq!(parsed.items[1].id, 102);
        assert_eq!(parsed.items[1].suggested_title, "Market Highs");
        assert_eq!(parsed.items[1].brief, "Global stock indices rally to record levels.");
    }

    #[test]
    fn test_parse_batch_response() {
        // 1. Direct valid JSON
        let valid_json = r#"{"top":"Top tech news today","items":[{"i":101,"t":"New AI Model","b":"Fast inference model released."}]}"#;
        let parsed = LlmCliRunner::parse_batch_response(valid_json);
        assert_eq!(parsed.top, "Top tech news today");
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].id, 101);
        assert_eq!(parsed.items[0].suggested_title, "New AI Model");
        assert_eq!(parsed.items[0].brief, "Fast inference model released.");

        // 2. Markdown fenced code block with conversational preamble
        let fenced = r#"
Here is the executive summary in JSON:
```json
{
  "top": "Market rallies",
  "items": [
    {"i": 202, "t": "Stocks Surge", "b": "Index hits record high."}
  ]
}
```
Hope this helps!
        "#;
        let parsed_fenced = LlmCliRunner::parse_batch_response(fenced);
        assert_eq!(parsed_fenced.top, "Market rallies");
        assert_eq!(parsed_fenced.items.len(), 1);
        assert_eq!(parsed_fenced.items[0].id, 202);
        assert_eq!(parsed_fenced.items[0].suggested_title, "Stocks Surge");

        // 3. Fallback regex parsing when JSON is slightly broken or surrounded by text without fences
        let raw_snippet = r#"
Some intro text
{"i": 303, "t": "Fallback Regex Title", "b": "Fallback brief content."}
Some outro text
        "#;
        let parsed_fallback = LlmCliRunner::parse_batch_response(raw_snippet);
        assert_eq!(parsed_fallback.items.len(), 1);
        assert_eq!(parsed_fallback.items[0].id, 303);
        assert_eq!(parsed_fallback.items[0].suggested_title, "Fallback Regex Title");
        assert_eq!(parsed_fallback.items[0].brief, "Fallback brief content.");

        // 4. Complete garbage output -> returns empty BatchAiResponse without crashing
        let garbage = "Totally invalid random response without any json";
        let parsed_garbage = LlmCliRunner::parse_batch_response(garbage);
        assert!(parsed_garbage.top.is_empty());
        assert!(parsed_garbage.items.is_empty());
    }

    #[test]
    fn test_llm_cli_prompt_formatting_and_args() {
        let runner = LlmCliRunner::new(
            "llm".to_string(),
            "gpt-4o-mini".to_string(),
            vec!["--thinking-budget".to_string(), "0".to_string()],
        );

        let prompt = LlmCliRunner::format_prompt("Rust 2026 Released", "Exciting new features landed.");
        assert!(prompt.contains("under 35 words"));
        assert!(prompt.contains("Article Title: Rust 2026 Released"));
        assert!(prompt.contains("Article Content:\nExciting new features landed."));

        let args = runner.build_args(&prompt);
        assert_eq!(args[0], "-m");
        assert_eq!(args[1], "gpt-4o-mini");
        assert_eq!(args[2], "--thinking-budget");
        assert_eq!(args[3], "0");
        assert_eq!(args[4], prompt);
    }

    #[tokio::test]
    async fn test_llm_cli_fallback_to_extractive_summary_on_failure() {
        // Points to non-existent command to simulate failure
        let runner = LlmCliRunner::new(
            "non-existent-llm-binary-xyz".to_string(),
            "gpt-4o-mini".to_string(),
            vec![],
        );

        let content = "First sentence of news. Second sentence of importance. Third sentence ignored.";
        let (summary, model) = runner.summarize("Tech Update", content).await;

        assert_eq!(model, "extractive-fallback");
        assert_eq!(summary, "First sentence of news. Second sentence of importance.");
    }

    #[tokio::test]
    async fn test_llm_cli_concurrency_throttling() {
        let runner = LlmCliRunner::new(
            "sh".to_string(),
            "test-model".to_string(),
            vec![],
        );

        // Verify semaphore starts with exactly 2 permits
        assert_eq!(runner.semaphore.available_permits(), 2);

        // Acquire permits to test bounding
        let p1 = runner.semaphore.acquire().await.unwrap();
        assert_eq!(runner.semaphore.available_permits(), 1);

        let p2 = runner.semaphore.acquire().await.unwrap();
        assert_eq!(runner.semaphore.available_permits(), 0);

        drop(p1);
        assert_eq!(runner.semaphore.available_permits(), 1);

        drop(p2);
        assert_eq!(runner.semaphore.available_permits(), 2);
    }

    #[test]
    fn test_server_logging_format() {
        let prompt = "Prompt text for testing";
        let dispatch_log = LlmCliRunner::format_dispatch_log("gemini-2.5-flash", 5, prompt);
        assert!(dispatch_log.starts_with("[AI_DISPATCH]"));
        assert!(dispatch_log.contains("model=\"gemini-2.5-flash\""));
        assert!(dispatch_log.contains("item_count=5"));
        assert!(dispatch_log.contains("prompt_preview=\"Prompt text for testing\""));

        let return_log = LlmCliRunner::format_return_log(123, "{\"top\":\"done\"}");
        assert!(return_log.starts_with("[AI_RETURN]"));
        assert!(return_log.contains("elapsed_ms=123"));
        assert!(return_log.contains("raw_output=\"{\\\"top\\\":\\\"done\\\"}\""));

        let warn_log = LlmCliRunner::format_warn_fallback("json_parse_fallback", "regex", "{\"i\":1}");
        assert!(warn_log.starts_with("[AI_WARN]"));
        assert!(warn_log.contains("reason=\"json_parse_fallback\""));
        assert!(warn_log.contains("fallback_strategy=\"regex\""));

        let timeout_log = LlmCliRunner::format_error_timeout(10, 10005);
        assert!(timeout_log.starts_with("[AI_ERROR]"));
        assert!(timeout_log.contains("error=\"cli_timeout\""));
        assert!(timeout_log.contains("timeout_secs=10"));
        assert!(timeout_log.contains("elapsed_ms=10005"));

        let proc_err_log = LlmCliRunner::format_error_process_failed(Some(1), "command failed");
        assert!(proc_err_log.starts_with("[AI_ERROR]"));
        assert!(proc_err_log.contains("error=\"process_failed\""));
        assert!(proc_err_log.contains("exit_code=Some(1)"));
        assert!(proc_err_log.contains("stderr=\"command failed\""));

        let empty_log = LlmCliRunner::format_error_empty_output(45);
        assert!(empty_log.starts_with("[AI_ERROR]"));
        assert!(empty_log.contains("error=\"empty_output\""));
        assert!(empty_log.contains("elapsed_ms=45"));
    }
}
