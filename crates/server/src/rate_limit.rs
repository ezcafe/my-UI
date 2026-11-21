use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct ClientBucket {
    tokens: f64,
    last_refill: Instant,
}

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, ClientBucket>>>,
    capacity: f64,
    refill_per_sec: f64,
}

impl RateLimiter {
    pub fn new(capacity: f64, refill_per_sec: f64) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            capacity,
            refill_per_sec,
        }
    }

    /// Checks rate limit for given IP.
    /// Returns Ok((limit, remaining)) if allowed, or Err(retry_after_seconds) if throttled.
    pub async fn check(&self, ip: &str) -> Result<(u64, u64), u64> {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        let bucket = buckets.entry(ip.to_string()).or_insert_with(|| ClientBucket {
            tokens: self.capacity,
            last_refill: now,
        });

        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            let remaining = bucket.tokens.floor() as u64;
            Ok((self.capacity as u64, remaining))
        } else {
            let wait_secs = ((1.0 - bucket.tokens) / self.refill_per_sec).ceil() as u64;
            Err(wait_secs.max(1))
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(20.0, 1.0)
    }
}

pub async fn rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Response {
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "127.0.0.1".to_string());

    match limiter.check(&ip).await {
        Ok((limit, remaining)) => {
            let mut response = next.run(req).await;
            response.headers_mut().insert(
                "X-RateLimit-Limit",
                HeaderValue::from_str(&limit.to_string()).unwrap(),
            );
            response.headers_mut().insert(
                "X-RateLimit-Remaining",
                HeaderValue::from_str(&remaining.to_string()).unwrap(),
            );
            response
        }
        Err(retry_after) => {
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                "Too Many Requests: Rate limit exceeded",
            )
                .into_response();
            response.headers_mut().insert(
                "Retry-After",
                HeaderValue::from_str(&retry_after.to_string()).unwrap(),
            );
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::build_app;
    use crate::config::Config;
    use axum::body::Body;
    use axum::http::StatusCode;
    use miniflux_client::client::MinifluxClient;
    use miniflux_client::schema::build_schema;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_rate_limiter_allows_burst_and_throttles_excess_requests() {
        let limiter = RateLimiter::new(20.0, 1.0);
        let ip = "192.168.1.1";

        // First 20 requests should be allowed (burst)
        for _ in 0..20 {
            assert!(limiter.check(ip).await.is_ok(), "Request within burst should succeed");
        }

        // 21st request should be throttled
        let excess = limiter.check(ip).await;
        assert!(excess.is_err(), "Excess request should be throttled");
    }

    #[tokio::test]
    async fn test_ddos_body_limit_rejects_payloads_exceeding_cap() {
        let config = Arc::new(Config {
            host: "127.0.0.1".to_string(),
            port: 3000,
            app_base_url: "http://127.0.0.1:3000".to_string(),
            auth_enabled: false,
            session_secret: "super-secret-key-that-is-at-least-32-bytes-long!".to_string(),
            oidc_issuer_url: "https://id.example.com".to_string(),
            oidc_client_id: "test-client".to_string(),
            oidc_client_secret: "test-secret".to_string(),
            oidc_redirect_url: "http://127.0.0.1:3000/auth/callback".to_string(),
            miniflux_base_url: "https://miniflux.example.com".to_string(),
            miniflux_api_key: "api-key".to_string(),
            ai_enabled: false,
            llm_cli_cmd: "llm".to_string(),
            llm_model: "gpt-4o-mini".to_string(),
            llm_extra_args: vec![],
            ai_db_path: "data/summaries.db".to_string(),
        });
        let client = MinifluxClient::new(config.miniflux_base_url.clone(), config.miniflux_api_key.clone());
        let schema = build_schema(client);
        let app = build_app(config, schema);

        let large_payload = format!("entry_ids={}", "a".repeat(70 * 1024));
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/hx/mark-page-read")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(large_payload))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}
