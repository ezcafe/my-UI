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
        r#"\{[^{}]*?"i"\s*:\s*(\d+)[^{}]*?"t"\s*:\s*"([^"\\]*(?:\\.[^"\\]*)*)"[^{}]*?(?:"r"|"b")\s*:\s*"([^"\\]*(?:\\.[^"\\]*)*)"[^{}]*?\}"#,
    )
    .expect("valid item regex")
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BatchAiItem {
    #[serde(rename = "i")]
    pub id: i64,
    #[serde(rename = "t")]
    pub suggested_title: String,
    #[serde(rename = "b", default)]
    pub brief: String,
    #[serde(rename = "r", default)]
    pub rewrite: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BatchAiResponse {
    #[serde(default)]
    pub top: String,
    #[serde(default)]
    pub items: Vec<BatchAiItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchArticleInput<'a> {
    pub id: i64,
    pub title: &'a str,
    pub source: &'a str,
    pub published_at: &'a str,
    pub content_or_brief: &'a str,
    pub is_cached: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
pub struct DetailAnalysisResult {
    pub rewrite: String,
    pub impact: String,
    pub fallacies: String,
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
            "Summarize what changed or why this news matters in strictly 3 to 5 concise sentences. Do not include introductory filler.\n\nArticle Title: {}\nArticle Content:\n{}",
            title.trim(),
            content.trim()
        )
    }

    pub fn format_batch_prompt_structured(items: &[BatchArticleInput<'_>]) -> String {
        let mut prompt = String::from(
            "You are an executive news editor. For the articles below, provide:\n\
             1. \"top\": A concise executive brief synthesizing the most important themes across all sources on this page (strictly 3 to 5 sentences).\n\
             2. \"items\": For each NEW article id, provide a punchy \"t\" (suggested title, strictly in one sentence) and \"r\" (inverted pyramid rewrite in strictly 3 to 5 sentences: lead with outcome and key impacts in first 2 sentences, followed by 2-3 bullet points with essential context). Do not regenerate items for existing articles.\n\n\
             REWRITE RULES FOR \"r\":\n\
             - Length: Strictly 3 to 5 sentences, under 110 words total (sweet spot 30-60 words).\n\
             - Structure: Inverted pyramid structure — lead with outcome and key impacts in first 2 sentences, followed by 2-3 bullet points with essential context.\n\
             - Tone & Style: Plain and simple language. Strictly omit introductory filler or closing boilerplate.\n\n\
             CRITICAL MULTILINGUAL RULE:\n\
             For each article and for the overall brief, write your response in the EXACT SAME LANGUAGE as the source text.\n\
             If an article is in Vietnamese, its suggested title and rewrite MUST be in Vietnamese.\n\
             If an article is in English, its suggested title and rewrite MUST be in English.\n\
             Do NOT translate articles into a different language.\n\n\
             Output raw JSON only using this compact schema:\n\
             {\"top\":\"...\",\"items\":[{\"i\":101,\"t\":\"...\",\"r\":\"...\"}]}\n\n\
             Articles by Source:\n\n",
        );

        let mut groups: Vec<(&str, Vec<&BatchArticleInput<'_>>)> = Vec::new();
        for item in items {
            let src = if item.source.trim().is_empty() {
                "General"
            } else {
                item.source.trim()
            };
            if let Some((_, list)) = groups.iter_mut().find(|(s, _)| *s == src) {
                list.push(item);
            } else {
                groups.push((src, vec![item]));
            }
        }

        for (src, mut group_items) in groups {
            prompt.push_str(&format!("### Source: {}\n", src));
            group_items.sort_by(|a, b| b.published_at.cmp(a.published_at));
            for item in group_items {
                let trimmed_title = item.title.trim();
                let mut clean_content = item.content_or_brief.trim();
                if let Some(stripped) =
                    clean_content.strip_prefix(&format!("Title: {}\n\n", trimmed_title))
                {
                    clean_content = stripped.trim();
                } else if let Some(stripped) =
                    clean_content.strip_prefix(&format!("Title: {}", trimmed_title))
                {
                    clean_content = stripped.trim();
                } else if let Some(stripped) = clean_content.strip_prefix("Title:") {
                    if let Some((_, rest)) = stripped.split_once("\n\n") {
                        clean_content = rest.trim();
                    }
                }
                if item.is_cached {
                    prompt.push_str(&format!(
                        "[ID {}] (Existing Brief) Title: {}\nRewrite:\n{}\n\n",
                        item.id, trimmed_title, clean_content
                    ));
                } else {
                    prompt.push_str(&format!(
                        "[ID {}] (New Article) Title: {}\nContent:\n{}\n\n",
                        item.id, trimmed_title, clean_content
                    ));
                }
            }
        }
        prompt
    }

    pub fn format_batch_prompt(articles: &[(i64, &str, &str)]) -> String {
        let items: Vec<BatchArticleInput> = articles
            .iter()
            .map(|(id, title, content)| BatchArticleInput {
                id: *id,
                title,
                source: "General",
                published_at: "",
                content_or_brief: content,
                is_cached: false,
            })
            .collect();
        Self::format_batch_prompt_structured(&items)
    }

    pub fn format_rewrite_and_analyze_prompt(title: &str, content: &str) -> String {
        format!(
            "Analyze and rewrite the following article. Output raw JSON only matching this schema:\n\
            {{\n  \
              \"rewrite\": \"Inverted pyramid rewrite: strictly under 110 words total (sweet spot 30-60 words). Lead with outcome and key impacts in first 2 sentences, followed by 2-3 bullet points with essential context. Plain language, no introductory filler, no closing boilerplate.\",\n  \
              \"impact\": \"Analyze the practical consequences and real-world impact of this news (who is affected, industry or social significance, short/long-term stakes).\",\n  \
              \"fallacies\": \"Identify any informal logical fallacies or rhetorical biases in the text (e.g. ad hominem, straw man, false dilemma, hasty generalization, cherry-picking, emotional appeal). If the reporting is objective, factual, and logically sound, strictly output: 'Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu' (for Vietnamese) or 'No logical fallacies detected — Objective and well-supported reporting' (for English).\"\n\
            }}\n\n\
            CRITICAL MULTILINGUAL RULE:\n\
            Write your response in the EXACT SAME LANGUAGE as the source article text.\n\
            If an article is in Vietnamese, all fields MUST be in Vietnamese.\n\
            If an article is in English, all fields MUST be in English.\n\
            Do NOT translate articles into a different language.\n\n\
            Output raw JSON only.\n\n\
            Title: {}\n\
            Content:\n{}",
            title.trim(),
            content.trim()
        )
    }

    pub fn parse_rewrite_and_analyze_response(raw_output: &str) -> DetailAnalysisResult {
        let trimmed = raw_output.trim();
        let mut parsed: Option<DetailAnalysisResult> = serde_json::from_str(trimmed).ok();

        if parsed.is_none() {
            if let Some(start) = trimmed.find("```") {
                let after_start = &trimmed[start + 3..];
                let code_content = if let Some(first_newline) = after_start.find('\n') {
                    &after_start[first_newline + 1..]
                } else {
                    after_start
                };
                if let Some(end) = code_content.rfind("```") {
                    let candidate = code_content[..end].trim();
                    parsed = serde_json::from_str(candidate).ok();
                }
            }
        }

        let mut result = parsed.unwrap_or_else(|| DetailAnalysisResult {
            rewrite: trimmed.to_string(),
            impact: String::new(),
            fallacies: String::new(),
        });

        // Normalize fallacies reassuring indicator
        let f_lower = result.fallacies.to_lowercase();
        let is_no_fallacy = f_lower.is_empty()
            || f_lower.contains("không phát hiện")
            || f_lower.contains("không có")
            || f_lower == "none"
            || f_lower.starts_with("none")
            || f_lower.contains("no fallacy")
            || f_lower.contains("no logical fallacies")
            || f_lower.contains("objective");

        if is_no_fallacy {
            let is_vietnamese = result
                .rewrite
                .chars()
                .any(|c| matches!(c, 'à'..='ỹ' | 'À'..='Ỹ'))
                || result
                    .impact
                    .chars()
                    .any(|c| matches!(c, 'à'..='ỹ' | 'À'..='Ỹ'))
                || f_lower.contains("không");
            if is_vietnamese {
                result.fallacies =
                    "Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu"
                        .to_string();
            } else {
                result.fallacies =
                    "No logical fallacies detected — Objective and well-supported reporting"
                        .to_string();
            }
        }

        result
    }

    pub fn format_impact_and_fallacies_prompt(title: &str, content: &str) -> String {
        format!(
            "Analyze the following article for real-world impact and logical fallacies. Output raw JSON only matching this schema:\n\
            {{\n  \
              \"impact\": \"Analyze the practical consequences and real-world impact of this news (who is affected, industry or social significance, short/long-term stakes).\",\n  \
              \"fallacies\": \"Identify any informal logical fallacies or rhetorical biases in the text (e.g. ad hominem, straw man, false dilemma, hasty generalization, cherry-picking, emotional appeal). If the reporting is objective, factual, and logically sound, strictly output: 'Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu' (for Vietnamese) or 'No logical fallacies detected — Objective and well-supported reporting' (for English).\"\n\
            }}\n\n\
            CRITICAL MULTILINGUAL RULE:\n\
            Write your response in the EXACT SAME LANGUAGE as the source article text.\n\
            If an article is in Vietnamese, all fields MUST be in Vietnamese.\n\
            If an article is in English, all fields MUST be in English.\n\
            Do NOT translate articles into a different language.\n\n\
            Output raw JSON only.\n\n\
            Title: {}\n\
            Content:\n{}",
            title.trim(),
            content.trim()
        )
    }

    pub fn parse_impact_and_fallacies_response(raw_output: &str) -> (String, String) {
        #[derive(Deserialize)]
        struct RawAnalysis {
            #[serde(default)]
            impact: String,
            #[serde(default)]
            fallacies: String,
        }

        let trimmed = raw_output.trim();
        let mut parsed: Option<RawAnalysis> = serde_json::from_str(trimmed).ok();

        if parsed.is_none() {
            if let Some(start) = trimmed.find("```") {
                let after_start = &trimmed[start + 3..];
                let code_content = if let Some(first_newline) = after_start.find('\n') {
                    &after_start[first_newline + 1..]
                } else {
                    after_start
                };
                if let Some(end) = code_content.rfind("```") {
                    let candidate = code_content[..end].trim();
                    parsed = serde_json::from_str(candidate).ok();
                }
            }
        }

        let raw = parsed.unwrap_or(RawAnalysis {
            impact: String::new(),
            fallacies: String::new(),
        });

        let impact = raw.impact;
        let mut fallacies = raw.fallacies;

        let f_lower = fallacies.to_lowercase();
        let is_no_fallacy = f_lower.is_empty()
            || f_lower.contains("không phát hiện")
            || f_lower.contains("không có")
            || f_lower == "none"
            || f_lower.starts_with("none")
            || f_lower.contains("no fallacy")
            || f_lower.contains("no logical fallacies")
            || f_lower.contains("objective");

        if is_no_fallacy {
            let is_vietnamese = impact.chars().any(|c| matches!(c, 'à'..='ỹ' | 'À'..='Ỹ'))
                || f_lower.contains("không");
            if is_vietnamese {
                fallacies =
                    "Không phát hiện lỗi ngụy biện — Lập luận khách quan, có số liệu đối chiếu"
                        .to_string();
            } else {
                fallacies =
                    "No logical fallacies detected — Objective and well-supported reporting"
                        .to_string();
            }
        }

        (impact, fallacies)
    }

    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec!["-m".to_string(), self.model.clone()];
        args.extend(self.extra_args.clone());
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
        if let Ok(mut res) = serde_json::from_str::<BatchAiResponse>(trimmed) {
            for item in &mut res.items {
                if item.brief.is_empty() && !item.rewrite.is_empty() {
                    item.brief = item.rewrite.clone();
                } else if item.rewrite.is_empty() && !item.brief.is_empty() {
                    item.rewrite = item.brief.clone();
                }
            }
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
                if let Ok(mut res) = serde_json::from_str::<BatchAiResponse>(candidate) {
                    for item in &mut res.items {
                        if item.brief.is_empty() && !item.rewrite.is_empty() {
                            item.brief = item.rewrite.clone();
                        } else if item.rewrite.is_empty() && !item.brief.is_empty() {
                            item.rewrite = item.brief.clone();
                        }
                    }
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
            if let (Some(id_str), Some(title), Some(content)) = (cap.get(1), cap.get(2), cap.get(3))
            {
                if let Ok(id) = id_str.as_str().parse::<i64>() {
                    let t = title.as_str().replace(r#"\""#, "\"");
                    let c = content.as_str().replace(r#"\""#, "\"");
                    items.push(BatchAiItem {
                        id,
                        suggested_title: t,
                        rewrite: c.clone(),
                        brief: c,
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

    pub async fn summarize_batch_structured(
        &self,
        articles: &[BatchArticleInput<'_>],
    ) -> (BatchAiResponse, String) {
        if articles.is_empty() {
            return (BatchAiResponse::default(), self.model.clone());
        }

        let _permit = self.semaphore.acquire().await.ok();
        let prompt = Self::format_batch_prompt_structured(articles);
        let args = self.build_args();

        let dispatch_msg = Self::format_dispatch_log(&self.model, articles.len(), &prompt);
        println!("{}", dispatch_msg);
        tracing::info!(
            target: "ai::batch",
            "{}",
            dispatch_msg
        );

        let start = std::time::Instant::now();
        const TIMEOUT_SECS: u64 = 90;
        let prompt_bytes = prompt.into_bytes();
        let result = tokio::time::timeout(Duration::from_secs(TIMEOUT_SECS), async {
            let mut cmd = Command::new(&self.cli_cmd);
            cmd.args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);

            let mut child = cmd.spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let _ = stdin.write_all(&prompt_bytes).await;
                drop(stdin);
            }

            child.wait_with_output().await
        })
        .await;

        match result {
            Ok(Ok(output)) => {
                let elapsed = start.elapsed().as_millis();
                if !output.status.success() {
                    let stderr_preview = String::from_utf8_lossy(&output.stderr);
                    let err_log =
                        Self::format_error_process_failed(output.status.code(), &stderr_preview);
                    eprintln!("{}", err_log);
                    tracing::error!(
                        target: "ai::batch",
                        "{}",
                        err_log
                    );
                    Self::fallback_extractive_structured(articles)
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
                        Self::fallback_extractive_structured(articles)
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
                Self::fallback_extractive_structured(articles)
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
                Self::fallback_extractive_structured(articles)
            }
        }
    }

    pub async fn summarize_batch(
        &self,
        articles: &[(i64, &str, &str)],
    ) -> (BatchAiResponse, String) {
        let items: Vec<BatchArticleInput> = articles
            .iter()
            .map(|(id, title, content)| BatchArticleInput {
                id: *id,
                title,
                source: "General",
                published_at: "",
                content_or_brief: content,
                is_cached: false,
            })
            .collect();
        self.summarize_batch_structured(&items).await
    }

    pub fn fallback_extractive_structured(
        articles: &[BatchArticleInput<'_>],
    ) -> (BatchAiResponse, String) {
        tracing::warn!(
            target: "ai::batch",
            "[AI_WARN] reason=\"cli_failure\" fallback_strategy=\"extractive\""
        );
        let mut items = Vec::new();
        for item in articles {
            if item.is_cached {
                items.push(BatchAiItem {
                    id: item.id,
                    suggested_title: String::new(),
                    rewrite: item.content_or_brief.to_string(),
                    brief: item.content_or_brief.to_string(),
                });
            } else {
                let fallback = miniflux_client::summary::extract_summary(item.content_or_brief, 3);
                let fallback_text = if fallback.is_empty() {
                    item.content_or_brief.trim().to_string()
                } else {
                    fallback
                };
                items.push(BatchAiItem {
                    id: item.id,
                    suggested_title: String::new(),
                    rewrite: fallback_text.clone(),
                    brief: fallback_text,
                });
            }
        }
        (
            BatchAiResponse {
                top: String::new(),
                items,
            },
            "extractive-fallback".to_string(),
        )
    }

    pub fn fallback_extractive(articles: &[(i64, &str, &str)]) -> (BatchAiResponse, String) {
        let items: Vec<BatchArticleInput> = articles
            .iter()
            .map(|(id, title, content)| BatchArticleInput {
                id: *id,
                title,
                source: "General",
                published_at: "",
                content_or_brief: content,
                is_cached: false,
            })
            .collect();
        Self::fallback_extractive_structured(&items)
    }

    pub async fn summarize(&self, title: &str, content: &str) -> (String, String) {
        let _permit = self.semaphore.acquire().await.ok();

        let prompt = Self::format_prompt(title, content);
        let args = self.build_args();

        let dispatch_msg = format!(
            "[AI_DISPATCH_SINGLE] model={} title=\"{}\" prompt_preview=\"{}\"",
            self.model,
            title.trim(),
            prompt
                .chars()
                .take(150)
                .collect::<String>()
                .replace('\n', " ")
        );
        println!("{}", dispatch_msg);
        tracing::info!(target: "ai::single", "{}", dispatch_msg);

        let start = std::time::Instant::now();
        const TIMEOUT_SECS: u64 = 30;
        let prompt_bytes = prompt.into_bytes();
        let result = tokio::time::timeout(Duration::from_secs(TIMEOUT_SECS), async {
            let mut cmd = Command::new(&self.cli_cmd);
            cmd.args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);

            let mut child = cmd.spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let _ = stdin.write_all(&prompt_bytes).await;
                drop(stdin);
            }

            child.wait_with_output().await
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
                    let fallback = miniflux_client::summary::extract_summary(content, 3);
                    let fallback_text = if fallback.is_empty() {
                        content.trim().to_string()
                    } else {
                        fallback
                    };
                    (fallback_text, "extractive-fallback".to_string())
                } else {
                    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if text.is_empty() {
                        let err_log =
                            format!("[AI_ERROR_SINGLE] empty_output elapsed_ms={}", elapsed);
                        eprintln!("{}", err_log);
                        tracing::error!(target: "ai::single", "{}", err_log);
                        let fallback = miniflux_client::summary::extract_summary(content, 3);
                        let fallback_text = if fallback.is_empty() {
                            content.trim().to_string()
                        } else {
                            fallback
                        };
                        (fallback_text, "extractive-fallback".to_string())
                    } else {
                        let return_msg = format!(
                            "[AI_RETURN_SINGLE] elapsed_ms={} output=\"{}\"",
                            elapsed,
                            text.chars()
                                .take(200)
                                .collect::<String>()
                                .replace('\n', " ")
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
                let fallback = miniflux_client::summary::extract_summary(content, 3);
                let fallback_text = if fallback.is_empty() {
                    content.trim().to_string()
                } else {
                    fallback
                };
                (fallback_text, "extractive-fallback".to_string())
            }
            Err(_) => {
                let elapsed = start.elapsed().as_millis();
                let err_log = format!(
                    "[AI_ERROR_SINGLE] timeout_after_{}s elapsed_ms={}",
                    TIMEOUT_SECS, elapsed
                );
                eprintln!("{}", err_log);
                tracing::error!(target: "ai::single", "{}", err_log);
                let fallback = miniflux_client::summary::extract_summary(content, 3);
                let fallback_text = if fallback.is_empty() {
                    content.trim().to_string()
                } else {
                    fallback
                };
                (fallback_text, "extractive-fallback".to_string())
            }
        }
    }

    pub async fn rewrite_and_analyze(
        &self,
        title: &str,
        content: &str,
    ) -> (DetailAnalysisResult, String) {
        let _permit = self.semaphore.acquire().await.ok();
        let prompt = Self::format_rewrite_and_analyze_prompt(title, content);
        let args = self.build_args();
        const TIMEOUT_SECS: u64 = 30;
        let prompt_bytes = prompt.into_bytes();
        let result = tokio::time::timeout(std::time::Duration::from_secs(TIMEOUT_SECS), async {
            let mut cmd = tokio::process::Command::new(&self.cli_cmd);
            cmd.args(&args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .kill_on_drop(true);

            let mut child = cmd.spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let _ = stdin.write_all(&prompt_bytes).await;
                drop(stdin);
            }

            child.wait_with_output().await
        })
        .await;

        match result {
            Ok(Ok(output)) if output.status.success() => {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if text.is_empty() {
                    (DetailAnalysisResult::default(), "error".to_string())
                } else {
                    let parsed = Self::parse_rewrite_and_analyze_response(&text);
                    (parsed, self.model.clone())
                }
            }
            _ => (DetailAnalysisResult::default(), "error".to_string()),
        }
    }

    pub async fn analyze_impact_and_fallacies(
        &self,
        title: &str,
        content: &str,
    ) -> ((String, String), String) {
        let _permit = self.semaphore.acquire().await.ok();
        let prompt = Self::format_impact_and_fallacies_prompt(title, content);
        let args = self.build_args();
        const TIMEOUT_SECS: u64 = 30;
        let prompt_bytes = prompt.into_bytes();
        let result = tokio::time::timeout(std::time::Duration::from_secs(TIMEOUT_SECS), async {
            let mut cmd = tokio::process::Command::new(&self.cli_cmd);
            cmd.args(&args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let _ = stdin.write_all(&prompt_bytes).await;
            }
            child.wait_with_output().await
        })
        .await;

        match result {
            Ok(Ok(output)) if output.status.success() => {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if text.is_empty() {
                    (
                        Self::parse_impact_and_fallacies_response(""),
                        "error".to_string(),
                    )
                } else {
                    let parsed = Self::parse_impact_and_fallacies_response(&text);
                    (parsed, self.model.clone())
                }
            }
            _ => (
                Self::parse_impact_and_fallacies_response(""),
                "error".to_string(),
            ),
        }
    }

    pub async fn rewrite(&self, title: &str, content: &str) -> (String, String) {
        let (res, model) = self.rewrite_and_analyze(title, content).await;
        (res.rewrite, model)
    }
}

#[derive(Clone)]
pub struct LlmHttpClient {
    client: reqwest::Client,
    pub provider: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
}

impl LlmHttpClient {
    pub fn new(provider: &str, api_key: &str, base_url: Option<&str>, model: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            provider: provider.to_lowercase(),
            api_key: api_key.to_string(),
            base_url: base_url.map(|s| s.to_string()),
            model: model.to_string(),
        }
    }

    pub async fn request_chat_completion(&self, prompt: &str) -> Result<String, String> {
        let is_gemini = self.provider == "gemini";
        if is_gemini {
            let base = self
                .base_url
                .as_deref()
                .unwrap_or("https://generativelanguage.googleapis.com/v1beta");
            let url = format!(
                "{}/models/{}:generateContent?key={}",
                base.trim_end_matches('/'),
                self.model,
                self.api_key
            );
            let payload = serde_json::json!({
                "contents": [
                    {
                        "parts": [
                            {"text": prompt}
                        ]
                    }
                ],
                "generationConfig": {
                    "responseMimeType": "application/json"
                }
            });

            let resp = self
                .client
                .post(&url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("Gemini API error ({}): {}", status, err_text));
            }

            let resp_json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let text = resp_json
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|cand| cand.get("content"))
                .and_then(|content| content.get("parts"))
                .and_then(|parts| parts.get(0))
                .and_then(|part| part.get("text"))
                .and_then(|t| t.as_str())
                .ok_or_else(|| {
                    "Missing candidates[0].content.parts[0].text in Gemini response".to_string()
                })?;

            Ok(text.to_string())
        } else {
            // OpenAI format
            let base = self
                .base_url
                .as_deref()
                .unwrap_or("https://api.openai.com/v1");
            let url = format!("{}/chat/completions", base.trim_end_matches('/'));
            let payload = serde_json::json!({
                "model": self.model,
                "messages": [
                    {"role": "user", "content": prompt}
                ],
                "response_format": {"type": "json_object"}
            });

            let resp = self
                .client
                .post(&url)
                .header(
                    reqwest::header::AUTHORIZATION,
                    format!("Bearer {}", self.api_key),
                )
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("OpenAI API error ({}): {}", status, err_text));
            }

            let resp_json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let text = resp_json
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|msg| msg.get("content"))
                .and_then(|content| content.as_str())
                .ok_or_else(|| {
                    "Missing choices[0].message.content in OpenAI response".to_string()
                })?;

            Ok(text.to_string())
        }
    }

    pub async fn summarize_batch_structured(
        &self,
        articles: &[BatchArticleInput<'_>],
    ) -> Result<(BatchAiResponse, String), String> {
        if articles.is_empty() {
            return Ok((BatchAiResponse::default(), self.model.clone()));
        }

        let prompt = LlmCliRunner::format_batch_prompt_structured(articles);
        let raw_output = self.request_chat_completion(&prompt).await?;
        let parsed = LlmCliRunner::parse_batch_response(&raw_output);
        Ok((parsed, self.model.clone()))
    }

    pub async fn summarize_batch(
        &self,
        articles: &[(i64, &str, &str)],
    ) -> Result<(BatchAiResponse, String), String> {
        let items: Vec<BatchArticleInput> = articles
            .iter()
            .map(|(id, title, content)| BatchArticleInput {
                id: *id,
                title,
                source: "General",
                published_at: "",
                content_or_brief: content,
                is_cached: false,
            })
            .collect();
        self.summarize_batch_structured(&items).await
    }

    pub async fn summarize(&self, title: &str, content: &str) -> Result<(String, String), String> {
        let prompt = LlmCliRunner::format_prompt(title, content);
        let raw_output = self.request_chat_completion(&prompt).await?;
        Ok((raw_output.trim().to_string(), self.model.clone()))
    }

    pub async fn analyze_impact_and_fallacies(
        &self,
        title: &str,
        content: &str,
    ) -> Result<((String, String), String), String> {
        let prompt = LlmCliRunner::format_impact_and_fallacies_prompt(title, content);
        let raw_output = self.request_chat_completion(&prompt).await?;
        let parsed = LlmCliRunner::parse_impact_and_fallacies_response(&raw_output);
        Ok((parsed, self.model.clone()))
    }
}

#[derive(Clone)]
pub enum LlmBackend {
    Cli(LlmCliRunner),
    Http {
        client: LlmHttpClient,
        fallback: LlmCliRunner,
    },
}

impl From<LlmCliRunner> for LlmBackend {
    fn from(cli: LlmCliRunner) -> Self {
        LlmBackend::Cli(cli)
    }
}

impl LlmBackend {
    pub async fn summarize_batch_structured(
        &self,
        articles: &[BatchArticleInput<'_>],
    ) -> (BatchAiResponse, String) {
        match self {
            Self::Cli(cli) => cli.summarize_batch_structured(articles).await,
            Self::Http { client, fallback } => {
                match client.summarize_batch_structured(articles).await {
                    Ok((res, model)) => (res, model),
                    Err(err) => {
                        tracing::warn!(target: "ai::llm", "HTTP LLM client failed: {}. Falling back to CLI", err);
                        fallback.summarize_batch_structured(articles).await
                    }
                }
            }
        }
    }

    pub async fn summarize_batch(
        &self,
        articles: &[(i64, &str, &str)],
    ) -> (BatchAiResponse, String) {
        match self {
            Self::Cli(cli) => cli.summarize_batch(articles).await,
            Self::Http { client, fallback } => match client.summarize_batch(articles).await {
                Ok((res, model)) => (res, model),
                Err(err) => {
                    tracing::warn!(target: "ai::llm", "HTTP LLM client failed: {}. Falling back to CLI", err);
                    fallback.summarize_batch(articles).await
                }
            },
        }
    }

    pub async fn summarize(&self, title: &str, content: &str) -> (String, String) {
        match self {
            Self::Cli(cli) => cli.summarize(title, content).await,
            Self::Http { client, fallback } => match client.summarize(title, content).await {
                Ok((res, model)) => (res, model),
                Err(err) => {
                    tracing::warn!(target: "ai::llm", "HTTP LLM client failed: {}. Falling back to CLI", err);
                    fallback.summarize(title, content).await
                }
            },
        }
    }

    pub async fn rewrite_and_analyze(
        &self,
        title: &str,
        content: &str,
    ) -> (DetailAnalysisResult, String) {
        match self {
            Self::Cli(cli) => cli.rewrite_and_analyze(title, content).await,
            Self::Http { fallback, .. } => fallback.rewrite_and_analyze(title, content).await,
        }
    }

    pub async fn rewrite(&self, title: &str, content: &str) -> (String, String) {
        match self {
            Self::Cli(cli) => cli.rewrite(title, content).await,
            Self::Http { fallback, .. } => fallback.rewrite(title, content).await,
        }
    }

    pub async fn analyze_impact_and_fallacies(
        &self,
        title: &str,
        content: &str,
    ) -> ((String, String), String) {
        match self {
            Self::Cli(cli) => cli.analyze_impact_and_fallacies(title, content).await,
            Self::Http { client, fallback } => {
                match client.analyze_impact_and_fallacies(title, content).await {
                    Ok(res) => res,
                    Err(err) => {
                        tracing::warn!(target: "ai::llm", "HTTP LLM client failed: {}. Falling back to CLI", err);
                        fallback.analyze_impact_and_fallacies(title, content).await
                    }
                }
            }
        }
    }

    pub async fn synthesize_page_brief_structured(
        &self,
        articles: &[BatchArticleInput<'_>],
    ) -> (String, String) {
        let (batch_res, model) = self.summarize_batch_structured(articles).await;
        (batch_res.top, model)
    }

    pub async fn synthesize_page_brief(&self, articles: &[(i64, &str, &str)]) -> (String, String) {
        let (batch_res, model) = self.summarize_batch(articles).await;
        (batch_res.top, model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_prompt_no_duplicate_titles() {
        let articles = [
            (
                101,
                "Article Title One",
                "Title: Article Title One\n\nBody content of article one.",
            ),
            (
                102,
                "Article Title Two",
                "Body content of article two without title prefix.",
            ),
        ];
        let prompt = LlmCliRunner::format_batch_prompt(&articles);

        // For article 101, Content should NOT repeat "Title: Article Title One"
        assert!(prompt.contains("[ID 101] (New Article) Title: Article Title One\nContent:\nBody content of article one."));
        assert!(!prompt.contains("Content:\nTitle: Article Title One"));

        // For article 102, Content is preserved cleanly
        assert!(prompt.contains("[ID 102] (New Article) Title: Article Title Two\nContent:\nBody content of article two without title prefix."));
    }

    #[tokio::test]
    async fn test_llm_http_client_openai_request_and_response() {
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let fake_response = r#"{
            "choices": [
                {
                    "message": {
                        "content": "{\"top\":\"Global Tech Summit Brief\",\"items\":[{\"i\":42,\"t\":\"AI Breakthrough\",\"b\":\"New reasoning models unveiled.\"}]}"
                    }
                }
            ]
        }"#;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("Authorization", "Bearer test-openai-key"))
            .respond_with(ResponseTemplate::new(200).set_body_string(fake_response))
            .mount(&mock_server)
            .await;

        let client = LlmHttpClient::new(
            "openai",
            "test-openai-key",
            Some(&format!("{}/v1", mock_server.uri())),
            "gpt-4o-mini",
        );

        let articles = [(42, "AI Breakthrough", "New models announced at summit.")];
        let (res, model) = client
            .summarize_batch(&articles)
            .await
            .expect("successful http call");

        assert_eq!(model, "gpt-4o-mini");
        assert_eq!(res.top, "Global Tech Summit Brief");
        assert_eq!(res.items.len(), 1);
        assert_eq!(res.items[0].id, 42);
        assert_eq!(res.items[0].suggested_title, "AI Breakthrough");
        assert_eq!(res.items[0].brief, "New reasoning models unveiled.");
    }

    #[tokio::test]
    async fn test_llm_http_client_gemini_request_and_response() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let fake_response = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "{\"top\":\"Gemini Fast Digest\",\"items\":[{\"i\":99,\"t\":\"Speed Increase\",\"b\":\"Inference latency dropped by 80%.\"}]}"
                            }
                        ]
                    }
                }
            ]
        }"#;

        Mock::given(method("POST"))
            .and(path("/models/gemini-2.5-flash:generateContent"))
            .respond_with(ResponseTemplate::new(200).set_body_string(fake_response))
            .mount(&mock_server)
            .await;

        let client = LlmHttpClient::new(
            "gemini",
            "test-gemini-key",
            Some(&mock_server.uri()),
            "gemini-2.5-flash",
        );

        let articles = [(99, "Speed Increase", "Latency dropped drastically today.")];
        let (res, model) = client
            .summarize_batch(&articles)
            .await
            .expect("successful gemini call");

        assert_eq!(model, "gemini-2.5-flash");
        assert_eq!(res.top, "Gemini Fast Digest");
        assert_eq!(res.items.len(), 1);
        assert_eq!(res.items[0].id, 99);
        assert_eq!(res.items[0].suggested_title, "Speed Increase");
        assert_eq!(res.items[0].brief, "Inference latency dropped by 80%.");
    }

    #[tokio::test]
    async fn test_llm_dispatcher_fallback_to_cli_on_http_failure() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        // Mock server returns 500 error
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let http_client = LlmHttpClient::new(
            "openai",
            "test-key",
            Some(&format!("{}/v1", mock_server.uri())),
            "gpt-4o-mini",
        );

        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        let script_content = "#!/bin/sh\nprintf '%s' '{\"top\":\"CLI Fallback Top\",\"items\":[{\"i\":77,\"t\":\"CLI Title\",\"b\":\"CLI Brief\"}]}'\n";
        script_file.write_all(script_content.as_bytes()).unwrap();
        let mut perms = std::fs::metadata(script_file.path()).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(script_file.path(), perms).unwrap();

        let cli_runner = LlmCliRunner::new(
            script_file.path().to_str().unwrap().to_string(),
            "fallback-cli-model".to_string(),
            vec![],
        );

        let backend = LlmBackend::Http {
            client: http_client,
            fallback: cli_runner,
        };

        let articles = [(77, "Test", "Content")];
        let (res, model) = backend.summarize_batch(&articles).await;

        assert_eq!(model, "fallback-cli-model");
        assert_eq!(res.top, "CLI Fallback Top");
        assert_eq!(res.items.len(), 1);
        assert_eq!(res.items[0].id, 77);
        assert_eq!(res.items[0].suggested_title, "CLI Title");
    }

    #[tokio::test]
    async fn test_llm_cli_pipes_prompt_via_stdin() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        let prompt_dump_file = tempfile::NamedTempFile::new().unwrap();
        let prompt_dump_path = prompt_dump_file.path().to_str().unwrap().to_string();

        let mut script_file = tempfile::NamedTempFile::new().unwrap();
        // The script reads stdin into the dump file and prints valid JSON response
        let script_content = format!(
            "#!/bin/sh\ncat > \"{}\"\nprintf '%s' '{{\"top\":\"OK\",\"items\":[]}}'\n",
            prompt_dump_path
        );
        script_file.write_all(script_content.as_bytes()).unwrap();
        let mut perms = std::fs::metadata(script_file.path()).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(script_file.path(), perms).unwrap();

        let runner = LlmCliRunner::new(
            script_file.path().to_str().unwrap().to_string(),
            "test-model".to_string(),
            vec![],
        );

        // Verify build_args does NOT contain prompt
        let args = runner.build_args();
        assert!(!args
            .iter()
            .any(|arg| arg.contains("Articles:") || arg.contains("executive news editor")));

        let articles = [(42, "Unique Stdin Title", "Content piped via stdin strictly")];
        let (res, _model) = runner.summarize_batch(&articles).await;
        assert_eq!(res.top, "OK");

        let piped_stdin = std::fs::read_to_string(&prompt_dump_path).unwrap();
        assert!(piped_stdin.contains("Unique Stdin Title"));
        assert!(piped_stdin.contains("Content piped via stdin strictly"));
    }

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
        assert!(prompt.contains(r#"{"top":"...","items":[{"i":101,"t":"...","r":"..."}]}"#));
        assert!(prompt.contains("[ID 101] (New Article) Title: Article 1"));
        assert!(prompt.contains("[ID 102] (New Article) Title: Article 2"));
        assert!(prompt.contains("Content of first article"));
        assert!(prompt.contains("Content of second article"));

        let args = runner.build_args();
        assert_eq!(args[0], "-m");
        assert_eq!(args[1], "gemini-2.5-flash");
        assert!(args.contains(&"--thinking-budget".to_string()));
        assert!(args.contains(&"0".to_string()));
        assert_eq!(args.len(), 4);
    }

    #[test]
    fn test_multilingual_batch_prompt_and_robust_parsing() {
        let articles = [
            (
                101,
                "Tin tức công nghệ",
                "Nội dung bài viết tiếng Việt về trí tuệ nhân tạo.",
            ),
            (
                102,
                "Global Markets",
                "Stock indexes reached all-time highs today.",
            ),
        ];
        let prompt = LlmCliRunner::format_batch_prompt(&articles);

        // Verify critical multilingual instructions are present in prompt
        assert!(prompt.contains("CRITICAL MULTILINGUAL RULE:"));
        assert!(prompt.contains("EXACT SAME LANGUAGE as the source text"));
        assert!(prompt.contains(
            "If an article is in Vietnamese, its suggested title and rewrite MUST be in Vietnamese."
        ));
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
        assert_eq!(
            parsed.top,
            "Tổng hợp tin tức công nghệ và thị trường tài chính quốc tế."
        );
        assert_eq!(parsed.items.len(), 2);
        assert_eq!(parsed.items[0].id, 101);
        assert_eq!(parsed.items[0].suggested_title, "Đột phá AI mới");
        assert_eq!(
            parsed.items[0].brief,
            "Công nghệ AI ghi nhận bước tiến quan trọng."
        );
        assert_eq!(parsed.items[1].id, 102);
        assert_eq!(parsed.items[1].suggested_title, "Market Highs");
        assert_eq!(
            parsed.items[1].brief,
            "Global stock indices rally to record levels."
        );
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
        assert_eq!(
            parsed_fallback.items[0].suggested_title,
            "Fallback Regex Title"
        );
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

        let prompt =
            LlmCliRunner::format_prompt("Rust 2026 Released", "Exciting new features landed.");
        assert!(prompt.contains("strictly 3 to 5 concise sentences"));
        assert!(prompt.contains("Article Title: Rust 2026 Released"));
        assert!(prompt.contains("Article Content:\nExciting new features landed."));

        let args = runner.build_args();
        assert_eq!(args[0], "-m");
        assert_eq!(args[1], "gpt-4o-mini");
        assert_eq!(args[2], "--thinking-budget");
        assert_eq!(args[3], "0");
        assert_eq!(args.len(), 4);
    }

    #[tokio::test]
    async fn test_llm_cli_fallback_to_extractive_summary_on_failure() {
        // Points to non-existent command to simulate failure
        let runner = LlmCliRunner::new(
            "non-existent-llm-binary-xyz".to_string(),
            "gpt-4o-mini".to_string(),
            vec![],
        );

        let content =
            "First sentence of news. Second sentence of importance. Third sentence ignored.";
        let (summary, model) = runner.summarize("Tech Update", content).await;

        assert_eq!(model, "extractive-fallback");
        assert_eq!(
            summary,
            "First sentence of news. Second sentence of importance. Third sentence ignored."
        );
    }

    #[tokio::test]
    async fn test_llm_cli_concurrency_throttling() {
        let runner = LlmCliRunner::new("sh".to_string(), "test-model".to_string(), vec![]);

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

        let warn_log =
            LlmCliRunner::format_warn_fallback("json_parse_fallback", "regex", "{\"i\":1}");
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

    #[test]
    fn test_format_prompt_requires_strictly_3_to_5_sentences() {
        let prompt = LlmCliRunner::format_prompt("Rust 2026", "Content of the article.");
        assert!(
            prompt.contains("strictly 3 to 5 concise sentences"),
            "Prompt should require strictly 3 to 5 concise sentences: {}",
            prompt
        );
        assert!(
            !prompt.contains("1 to 2"),
            "Prompt should not contain 1 to 2 sentences"
        );
    }

    #[test]
    fn test_format_batch_prompt_requires_strictly_3_to_5_sentences() {
        let articles = [(1, "Title", "Content")];
        let prompt = LlmCliRunner::format_batch_prompt(&articles);
        assert!(
            prompt.contains("strictly 3 to 5 sentences"),
            "Batch prompt should require strictly 3 to 5 sentences: {}",
            prompt
        );
    }

    #[test]
    fn test_format_batch_prompt_strictly_rewrite_structure() {
        let articles = [(1, "Title", "Content")];
        let prompt = LlmCliRunner::format_batch_prompt(&articles);
        assert!(
            prompt.contains("inverted pyramid rewrite"),
            "Batch prompt should require inverted pyramid rewrite: {}",
            prompt
        );
    }

    #[test]
    fn test_extractive_fallback_3_sentences() {
        let content = "Sentence one. Sentence two. Sentence three. Sentence four.";
        let articles = [(1, "Title", content)];
        let (res, model) = LlmCliRunner::fallback_extractive(&articles);
        assert_eq!(model, "extractive-fallback");
        assert_eq!(
            res.items[0].brief,
            "Sentence one. Sentence two. Sentence three."
        );
    }

    #[test]
    fn test_format_batch_prompt_source_grouping_and_date_sorting() {
        let items = [
            BatchArticleInput {
                id: 101,
                title: "Rust 1.84",
                source: "Rust Blog",
                published_at: "2026-11-20T10:00:00Z",
                content_or_brief: "Old rust content",
                is_cached: false,
            },
            BatchArticleInput {
                id: 102,
                title: "Rust 1.85",
                source: "Rust Blog",
                published_at: "2026-11-22T10:00:00Z",
                content_or_brief: "New rust content",
                is_cached: false,
            },
            BatchArticleInput {
                id: 201,
                title: "HN Story",
                source: "Hacker News",
                published_at: "2026-11-21T08:00:00Z",
                content_or_brief: "HN content",
                is_cached: false,
            },
        ];

        let prompt = LlmCliRunner::format_batch_prompt_structured(&items);
        assert!(
            prompt.contains("### Source: Rust Blog"),
            "Must contain Source header for Rust Blog"
        );
        assert!(
            prompt.contains("### Source: Hacker News"),
            "Must contain Source header for Hacker News"
        );

        let pos_102 = prompt.find("[ID 102]").expect("ID 102 present");
        let pos_101 = prompt.find("[ID 101]").expect("ID 101 present");
        assert!(
            pos_102 < pos_101,
            "Latest article 102 (Nov 22) must appear before older article 101 (Nov 20)"
        );
    }

    #[test]
    fn test_format_batch_prompt_handles_cached_briefs_and_new_articles() {
        let items = [
            BatchArticleInput {
                id: 101,
                title: "New Post",
                source: "Tech News",
                published_at: "2026-11-22T12:00:00Z",
                content_or_brief: "Full text of new post",
                is_cached: false,
            },
            BatchArticleInput {
                id: 102,
                title: "Cached Post",
                source: "Tech News",
                published_at: "2026-11-21T12:00:00Z",
                content_or_brief: "Brief already in db",
                is_cached: true,
            },
        ];

        let prompt = LlmCliRunner::format_batch_prompt_structured(&items);
        assert!(prompt.contains("[ID 101] (New Article) Title: New Post"));
        assert!(prompt.contains("Content:\nFull text of new post"));

        assert!(
            prompt.contains("[ID 102] (Existing Brief) Title: Cached Post")
                || prompt.contains("[ID 102] (Existing Article) Title: Cached Post")
        );
        assert!(
            prompt.contains("Rewrite:\nBrief already in db")
                || prompt.contains("Rewrite: Brief already in db")
        );

        assert!(
            prompt.contains("Do not regenerate items for existing articles")
                || prompt.contains("For each NEW article id")
        );
    }

    #[test]
    fn test_format_batch_prompt_suggested_title_one_sentence() {
        let items = [BatchArticleInput {
            id: 101,
            title: "New Post",
            source: "Tech News",
            published_at: "2026-11-22T12:00:00Z",
            content_or_brief: "Full text of new post",
            is_cached: false,
        }];
        let prompt = LlmCliRunner::format_batch_prompt_structured(&items);
        assert!(
            prompt.contains("suggested title, strictly in one sentence"),
            "Batch prompt must require suggested title strictly in one sentence: {}",
            prompt
        );
        assert!(
            !prompt.contains("<=10 words"),
            "Batch prompt must not contain '<=10 words': {}",
            prompt
        );
    }

    #[test]
    fn test_llm_rewrite_and_analyze_json_parsing_and_reassurance() {
        let prompt = LlmCliRunner::format_rewrite_and_analyze_prompt(
            "Tiêu đề bài viết",
            "Nội dung bài viết mẫu",
        );
        assert!(prompt.contains("\"rewrite\""));
        assert!(prompt.contains("\"impact\""));
        assert!(prompt.contains("\"fallacies\""));
        assert!(prompt.contains("CRITICAL MULTILINGUAL RULE"));

        // 1. Direct JSON parse with explicit fallacies
        let raw_json = r#"{
            "rewrite": "Lead sentence.\n\n• Bullet 1",
            "impact": "Tác động lớn đến thị trường.",
            "fallacies": "Ngụy biện khái quát hóa vội vã trong đoạn 2."
        }"#;
        let res = LlmCliRunner::parse_rewrite_and_analyze_response(raw_json);
        assert_eq!(res.rewrite, "Lead sentence.\n\n• Bullet 1");
        assert_eq!(res.impact, "Tác động lớn đến thị trường.");
        assert_eq!(
            res.fallacies,
            "Ngụy biện khái quát hóa vội vã trong đoạn 2."
        );

        // 2. Fenced markdown block with "Không phát hiện" -> normalized to reassuring indicator
        let fenced_json = "```json\n{\n  \"rewrite\": \"Inverted pyramid lead.\",\n  \"impact\": \"Economic shift.\",\n  \"fallacies\": \"Không phát hiện\"\n}\n```";
        let res2 = LlmCliRunner::parse_rewrite_and_analyze_response(fenced_json);
        assert_eq!(res2.rewrite, "Inverted pyramid lead.");
        assert_eq!(res2.impact, "Economic shift.");
        assert!(res2.fallacies.contains("Không phát hiện lỗi ngụy biện"));
        assert!(res2.fallacies.contains("Lập luận khách quan"));

        // 3. English "none" -> normalized to English reassuring indicator
        let english_json = r#"{
            "rewrite": "English lead sentence.",
            "impact": "Consumer impact.",
            "fallacies": "None"
        }"#;
        let res3 = LlmCliRunner::parse_rewrite_and_analyze_response(english_json);
        assert!(res3.fallacies.contains("No logical fallacies detected"));
        assert!(res3.fallacies.contains("Objective"));
    }

    #[test]
    fn test_format_batch_prompt_requires_rewrite_r() {
        let articles = [(1, "Title", "Content")];
        let prompt = LlmCliRunner::format_batch_prompt(&articles);
        assert!(
            prompt.contains(r#"{"top":"...","items":[{"i":101,"t":"...","r":"..."}]}"#),
            "Batch prompt compact schema must use 'r': {}",
            prompt
        );
        assert!(
            prompt.contains("inverted pyramid rewrite") || prompt.contains("rewrite"),
            "Batch prompt must require rewrite instead of brief: {}",
            prompt
        );
        assert!(
            !prompt.contains(r#"{"top":"...","items":[{"i":101,"t":"...","b":"..."}]}"#),
            "Batch prompt must NOT specify 'b' in schema: {}",
            prompt
        );
    }

    #[test]
    fn test_parse_batch_response_parses_rewrite_field() {
        let json = r#"{"top":"Synthesized overview","items":[{"i":101,"t":"AI Headline","r":"Lead sentence 1. Impact sentence 2.\n\n• Bullet 1"}]}"#;
        let parsed = LlmCliRunner::parse_batch_response(json);
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].id, 101);
        assert_eq!(parsed.items[0].suggested_title, "AI Headline");
        assert_eq!(
            parsed.items[0].rewrite,
            "Lead sentence 1. Impact sentence 2.\n\n• Bullet 1"
        );
    }

    #[test]
    fn test_parse_batch_response_fallback_regex_captures_rewrite() {
        let raw = r#"Here is the requested output:
        {"top": "Page synthesis", "items": [
            {"i": 202, "t": "Regex Title", "r": "Extracted rewrite text via regex."}
        ]}"#;
        let parsed = LlmCliRunner::parse_batch_response(raw);
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].id, 202);
        assert_eq!(parsed.items[0].suggested_title, "Regex Title");
        assert_eq!(parsed.items[0].rewrite, "Extracted rewrite text via regex.");
    }

    #[test]
    fn test_format_impact_and_fallacies_prompt_and_parser() {
        let prompt = LlmCliRunner::format_impact_and_fallacies_prompt("Test Title", "Test Content");
        assert!(prompt.contains("\"impact\""));
        assert!(prompt.contains("\"fallacies\""));
        assert!(!prompt.contains("\"rewrite\""));

        let raw = r#"{"impact": "Major supply chain shift.", "fallacies": "None"}"#;
        let parsed = LlmCliRunner::parse_impact_and_fallacies_response(raw);
        assert_eq!(parsed.0, "Major supply chain shift.");
        assert!(parsed.1.contains("No logical fallacies detected"));
    }

    #[test]
    fn test_batch_prompt_rewrite_rules_word_count_and_inverted_pyramid() {
        let articles = [(1, "Title", "Content")];
        let prompt = LlmCliRunner::format_batch_prompt(&articles);

        assert!(
            prompt.contains("under 110 words"),
            "Prompt must specify under 110 words: {}",
            prompt
        );
        assert!(
            prompt.contains("30-60 words") || prompt.contains("30–60 words"),
            "Prompt must specify 30-60 words sweet spot: {}",
            prompt
        );
        assert!(
            prompt.contains("inverted pyramid"),
            "Prompt must specify inverted pyramid: {}",
            prompt
        );
        assert!(
            prompt.contains("lead with outcome and key impacts in first 2 sentences")
                || prompt.contains("first 2 sentences"),
            "Prompt must require outcome in first 2 sentences: {}",
            prompt
        );
        assert!(
            prompt.contains("2-3 bullet points"),
            "Prompt must require 2-3 bullet points: {}",
            prompt
        );
        assert!(
            prompt.contains("no introductory filler")
                || prompt.contains("omit introductory filler"),
            "Prompt must forbid intro filler: {}",
            prompt
        );
        assert!(
            prompt.contains("closing") || prompt.contains("boilerplate"),
            "Prompt must forbid closing boilerplate: {}",
            prompt
        );
    }

    #[test]
    fn test_rewrite_and_analyze_prompt_rules() {
        let prompt = LlmCliRunner::format_rewrite_and_analyze_prompt("Test Title", "Test Content");
        let prompt_lower = prompt.to_lowercase();

        assert!(
            prompt_lower.contains("under 110 words"),
            "Prompt must specify under 110 words: {}",
            prompt
        );
        assert!(
            prompt_lower.contains("30-60 words") || prompt_lower.contains("30–60 words"),
            "Prompt must specify 30-60 words sweet spot: {}",
            prompt
        );
        assert!(
            prompt_lower.contains("inverted pyramid"),
            "Prompt must specify inverted pyramid: {}",
            prompt
        );
        assert!(
            prompt_lower.contains("2-3 bullet points"),
            "Prompt must require 2-3 bullet points: {}",
            prompt
        );
        assert!(
            prompt_lower.contains("no introductory filler")
                || prompt_lower.contains("omit introductory filler")
                || prompt_lower.contains("no filler"),
            "Prompt must forbid intro filler: {}",
            prompt
        );
        assert!(
            prompt_lower.contains("closing") || prompt_lower.contains("boilerplate"),
            "Prompt must forbid closing boilerplate: {}",
            prompt
        );
    }
}
