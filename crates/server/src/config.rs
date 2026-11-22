use std::collections::HashMap;
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingVar(String),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub app_base_url: String,
    pub auth_enabled: bool,
    pub session_secret: String,
    pub oidc_issuer_url: String,
    pub oidc_client_id: String,
    pub oidc_client_secret: String,
    pub oidc_redirect_url: String,
    pub miniflux_base_url: String,
    pub miniflux_api_key: String,
    pub ai_enabled: bool,
    pub llm_cli_cmd: String,
    pub llm_model: String,
    pub llm_extra_args: Vec<String>,
    pub ai_db_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::load_dotenv();
        let vars: HashMap<String, String> = env::vars().collect();
        Self::from_map(&vars)
    }

    pub fn load_dotenv() {
        if let Ok(content) = std::fs::read_to_string(".env") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, val)) = line.split_once('=') {
                    let key = key.trim();
                    let val = val.trim().trim_matches('"').trim_matches('\'');
                    if env::var(key).is_err() {
                        env::set_var(key, val);
                    }
                }
            }
        }
    }

    pub fn from_map(map: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let get_req = |key: &str| -> Result<String, ConfigError> {
            map.get(key)
                .cloned()
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| ConfigError::MissingVar(key.to_string()))
        };

        let port: u16 = match map.get("PORT") {
            Some(p) => p.parse().map_err(|_| ConfigError::InvalidValue("PORT must be a valid u16".to_string()))?,
            None => 3000,
        };

        let host = map.get("HOST").cloned().unwrap_or_else(|| "0.0.0.0".to_string());
        let app_base_url = map.get("APP_BASE_URL").cloned().unwrap_or_else(|| format!("http://{}:{}", host, port));

        let auth_enabled = map
            .get("AUTH_ENABLED")
            .map(|v| !v.eq_ignore_ascii_case("false") && v != "0")
            .unwrap_or(true);

        let session_secret = if auth_enabled {
            let secret = get_req("SESSION_SECRET")?;
            if secret.len() < 32 {
                return Err(ConfigError::InvalidValue(
                    "SESSION_SECRET must be at least 32 bytes long".to_string(),
                ));
            }
            secret
        } else {
            map.get("SESSION_SECRET")
                .cloned()
                .filter(|s| s.len() >= 32)
                .unwrap_or_else(|| "guest-mode-session-secret-at-least-32-chars-long!".to_string())
        };

        let oidc_issuer_url = if auth_enabled {
            get_req("OIDC_ISSUER_URL")?
        } else {
            map.get("OIDC_ISSUER_URL")
                .cloned()
                .unwrap_or_else(|| "https://auth-disabled.local".to_string())
        };

        let oidc_client_id = if auth_enabled {
            get_req("OIDC_CLIENT_ID")?
        } else {
            map.get("OIDC_CLIENT_ID")
                .cloned()
                .unwrap_or_else(|| "guest".to_string())
        };

        let oidc_client_secret = if auth_enabled {
            get_req("OIDC_CLIENT_SECRET")?
        } else {
            map.get("OIDC_CLIENT_SECRET")
                .cloned()
                .unwrap_or_else(|| "guest".to_string())
        };

        let oidc_redirect_url = if auth_enabled {
            get_req("OIDC_REDIRECT_URL")?
        } else {
            map.get("OIDC_REDIRECT_URL")
                .cloned()
                .unwrap_or_else(|| format!("{}/auth/callback", app_base_url))
        };

        let miniflux_base_url = get_req("MINIFLUX_BASE_URL")?;
        let miniflux_api_key = get_req("MINIFLUX_API_KEY")?;

        let ai_enabled = auth_enabled
            && map
                .get("AI_ENABLED")
                .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
                .unwrap_or(false);

        let llm_cli_cmd = map
            .get("LLM_CLI_CMD")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "llm".to_string());

        let llm_model = map
            .get("LLM_MODEL")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "gpt-4o-mini".to_string());

        let llm_extra_args = map
            .get("LLM_EXTRA_ARGS")
            .map(|s| s.split_whitespace().map(|item| item.to_string()).collect())
            .unwrap_or_default();

        let ai_db_path = map
            .get("AI_DB_PATH")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "data/summaries.db".to_string());

        Ok(Config {
            host,
            port,
            app_base_url,
            auth_enabled,
            session_secret,
            oidc_issuer_url,
            oidc_client_id,
            oidc_client_secret,
            oidc_redirect_url,
            miniflux_base_url,
            miniflux_api_key,
            ai_enabled,
            llm_cli_cmd,
            llm_model,
            llm_extra_args,
            ai_db_path,
        })
    }

    pub fn is_secure(&self) -> bool {
        self.app_base_url.starts_with("https://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_env(vars: &[(&str, &str)]) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (k, v) in vars {
            map.insert(k.to_string(), v.to_string());
        }
        map
    }

    #[test]
    fn test_config_loads_from_valid_env() {
        let vars = [
            ("PORT", "8080"),
            ("HOST", "127.0.0.1"),
            ("APP_BASE_URL", "http://127.0.0.1:8080"),
            ("SESSION_SECRET", "super-secret-key-that-is-at-least-32-bytes-long!"),
            ("OIDC_ISSUER_URL", "https://id.example.com"),
            ("OIDC_CLIENT_ID", "client-123"),
            ("OIDC_CLIENT_SECRET", "secret-456"),
            ("OIDC_REDIRECT_URL", "http://127.0.0.1:8080/auth/callback"),
            ("MINIFLUX_BASE_URL", "https://miniflux.example.com"),
            ("MINIFLUX_API_KEY", "miniflux-key-789"),
        ];
        let config = Config::from_map(&make_test_env(&vars)).expect("Failed to load valid config");
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.app_base_url, "http://127.0.0.1:8080");
        assert_eq!(config.session_secret, "super-secret-key-that-is-at-least-32-bytes-long!");
        assert_eq!(config.oidc_issuer_url, "https://id.example.com");
        assert_eq!(config.oidc_client_id, "client-123");
        assert_eq!(config.oidc_client_secret, "secret-456");
        assert_eq!(config.oidc_redirect_url, "http://127.0.0.1:8080/auth/callback");
        assert_eq!(config.miniflux_base_url, "https://miniflux.example.com");
        assert_eq!(config.miniflux_api_key, "miniflux-key-789");
    }

    #[test]
    fn test_config_fails_when_secret_too_short() {
        let vars = [
            ("SESSION_SECRET", "too-short"),
            ("OIDC_ISSUER_URL", "https://id.example.com"),
            ("OIDC_CLIENT_ID", "client-123"),
            ("OIDC_CLIENT_SECRET", "secret-456"),
            ("OIDC_REDIRECT_URL", "http://localhost:3000/auth/callback"),
            ("MINIFLUX_BASE_URL", "https://miniflux.example.com"),
            ("MINIFLUX_API_KEY", "miniflux-key"),
        ];
        let res = Config::from_map(&make_test_env(&vars));
        assert!(res.is_err(), "Expected error for short secret");
        match res.unwrap_err() {
            ConfigError::InvalidValue(msg) => assert!(msg.contains("SESSION_SECRET")),
            other => panic!("Unexpected error type: {:?}", other),
        }
    }

    #[test]
    fn test_config_defaults() {
        let vars = [
            ("SESSION_SECRET", "super-secret-key-that-is-at-least-32-bytes-long!"),
            ("OIDC_ISSUER_URL", "https://id.example.com"),
            ("OIDC_CLIENT_ID", "client-123"),
            ("OIDC_CLIENT_SECRET", "secret-456"),
            ("OIDC_REDIRECT_URL", "http://localhost:3000/auth/callback"),
            ("MINIFLUX_BASE_URL", "https://miniflux.example.com"),
            ("MINIFLUX_API_KEY", "miniflux-key"),
        ];
        let config = Config::from_map(&make_test_env(&vars)).expect("Failed to load config with defaults");
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.app_base_url, "http://0.0.0.0:3000");
    }

    #[test]
    fn test_config_parses_ai_settings_and_defaults() {
        let default_vars = [
            ("SESSION_SECRET", "super-secret-key-that-is-at-least-32-bytes-long!"),
            ("OIDC_ISSUER_URL", "https://id.example.com"),
            ("OIDC_CLIENT_ID", "client-123"),
            ("OIDC_CLIENT_SECRET", "secret-456"),
            ("OIDC_REDIRECT_URL", "http://localhost:3000/auth/callback"),
            ("MINIFLUX_BASE_URL", "https://miniflux.example.com"),
            ("MINIFLUX_API_KEY", "miniflux-key"),
        ];
        let def_config = Config::from_map(&make_test_env(&default_vars)).expect("defaults");
        assert!(!def_config.ai_enabled);
        assert_eq!(def_config.llm_cli_cmd, "llm");
        assert_eq!(def_config.llm_model, "gpt-4o-mini");
        assert!(def_config.llm_extra_args.is_empty());
        assert_eq!(def_config.ai_db_path, "data/summaries.db");

        let mut custom_vars = make_test_env(&default_vars);
        custom_vars.insert("AI_ENABLED".to_string(), "true".to_string());
        custom_vars.insert("LLM_CLI_CMD".to_string(), "my-llm".to_string());
        custom_vars.insert("LLM_MODEL".to_string(), "claude-3-5".to_string());
        custom_vars.insert("LLM_EXTRA_ARGS".to_string(), "--thinking-budget 0 --opt foo".to_string());
        custom_vars.insert("AI_DB_PATH".to_string(), "custom/db.sqlite".to_string());

        let custom_config = Config::from_map(&custom_vars).expect("custom");
        assert!(custom_config.ai_enabled);
        assert_eq!(custom_config.llm_cli_cmd, "my-llm");
        assert_eq!(custom_config.llm_model, "claude-3-5");
        assert_eq!(custom_config.llm_extra_args, vec!["--thinking-budget", "0", "--opt", "foo"]);
        assert_eq!(custom_config.ai_db_path, "custom/db.sqlite");
    }

    #[test]
    fn test_config_parses_auth_enabled_and_defaults() {
        let vars = [
            ("SESSION_SECRET", "super-secret-key-that-is-at-least-32-bytes-long!"),
            ("OIDC_ISSUER_URL", "https://id.example.com"),
            ("OIDC_CLIENT_ID", "client-123"),
            ("OIDC_CLIENT_SECRET", "secret-456"),
            ("OIDC_REDIRECT_URL", "http://localhost:3000/auth/callback"),
            ("MINIFLUX_BASE_URL", "https://miniflux.example.com"),
            ("MINIFLUX_API_KEY", "miniflux-key"),
        ];
        let def_config = Config::from_map(&make_test_env(&vars)).expect("defaults");
        assert!(def_config.auth_enabled, "auth_enabled should default to true");

        let mut false_vars = make_test_env(&vars);
        false_vars.insert("AUTH_ENABLED".to_string(), "false".to_string());
        let false_config = Config::from_map(&false_vars).expect("auth disabled");
        assert!(!false_config.auth_enabled);
    }

    #[test]
    fn test_config_allows_missing_oidc_when_auth_disabled() {
        let vars = [
            ("AUTH_ENABLED", "false"),
            ("MINIFLUX_BASE_URL", "https://miniflux.example.com"),
            ("MINIFLUX_API_KEY", "miniflux-key"),
        ];
        let config = Config::from_map(&make_test_env(&vars)).expect("should succeed without OIDC vars");
        assert!(!config.auth_enabled);
        assert!(!config.session_secret.is_empty());
        assert!(!config.oidc_issuer_url.is_empty());
    }

    #[test]
    fn test_config_ai_disabled_when_auth_disabled() {
        let vars = [
            ("AUTH_ENABLED", "false"),
            ("AI_ENABLED", "true"),
            ("MINIFLUX_BASE_URL", "https://miniflux.example.com"),
            ("MINIFLUX_API_KEY", "miniflux-key"),
        ];
        let config = Config::from_map(&make_test_env(&vars)).expect("should succeed");
        assert!(!config.auth_enabled);
        assert!(
            !config.ai_enabled,
            "ai_enabled must be false when auth_enabled is false even if AI_ENABLED=true"
        );
    }

    #[test]
    fn test_config_ai_matrix_permutations() {
        let base_vars = [
            ("MINIFLUX_BASE_URL", "https://miniflux.example.com"),
            ("MINIFLUX_API_KEY", "miniflux-key"),
            ("SESSION_SECRET", "super-secret-key-that-is-at-least-32-bytes-long!"),
            ("OIDC_ISSUER_URL", "https://id.example.com"),
            ("OIDC_CLIENT_ID", "client-123"),
            ("OIDC_CLIENT_SECRET", "secret-456"),
            ("OIDC_REDIRECT_URL", "http://localhost:3000/auth/callback"),
        ];

        let test_matrix = [
            ("false", "false", false),
            ("false", "true", false),
            ("true", "false", false),
            ("true", "true", true),
        ];

        for (auth_val, ai_val, expected_ai) in test_matrix {
            let mut map = make_test_env(&base_vars);
            map.insert("AUTH_ENABLED".to_string(), auth_val.to_string());
            map.insert("AI_ENABLED".to_string(), ai_val.to_string());

            let config = Config::from_map(&map).expect("config load");
            assert_eq!(
                config.ai_enabled, expected_ai,
                "Failed matrix permutation AUTH_ENABLED={}, AI_ENABLED={}: expected ai_enabled={}",
                auth_val, ai_val, expected_ai
            );
        }
    }
}
