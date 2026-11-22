use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Semaphore;

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

    pub fn build_args(&self, prompt: &str) -> Vec<String> {
        let mut args = vec!["-m".to_string(), self.model.clone()];
        args.extend(self.extra_args.clone());
        args.push(prompt.to_string());
        args
    }

    pub async fn summarize(&self, title: &str, content: &str) -> (String, String) {
        let _permit = self.semaphore.acquire().await.ok();

        let prompt = Self::format_prompt(title, content);
        let args = self.build_args(&prompt);

        let result = tokio::time::timeout(Duration::from_secs(5), async {
            let mut cmd = Command::new(&self.cli_cmd);
            cmd.args(&args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);

            let output = cmd.output().await.ok()?;
            if !output.status.success() {
                return None;
            }

            let text = String::from_utf8(output.stdout).ok()?;
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .await;

        match result {
            Ok(Some(ai_summary)) => (ai_summary, self.model.clone()),
            _ => {
                // Fallback to extractive summary
                let fallback = miniflux_client::summary::extract_summary(content, 2);
                let fallback_text = if fallback.is_empty() {
                    content.trim().to_string()
                } else {
                    fallback
                };
                (fallback_text, "extractive-fallback".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
