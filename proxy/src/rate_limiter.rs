// proxy/src/rate_limiter.rs
//
// Per-client-IP token bucket rate limiter for the Clean-CTX proxy.
//
// This is a separate concern from the global connection semaphore (MAX_CONNECTIONS).
// The semaphore controls how many connections exist *at once*. The rate limiter
// controls how many requests per second a single client can make.
//
// Uses a GC-enabled sliding-window design to avoid unbounded map growth:
// entries older than the GC interval are evicted on each check.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// A per-client-IP token bucket rate limiter.
///
/// Each client (identified by IP) gets a bucket that refills at `rps` tokens/second
/// up to a maximum burst of `burst` tokens. A request "costs" one token.
/// If no tokens remain, the request is rate-limited (429).
pub struct RateLimiter {
    inner: Mutex<Inner>,
    rps: f64,
    burst: f64,
}

struct Inner {
    buckets: HashMap<String, Bucket>,
    last_gc: Instant,
}

#[derive(Clone)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

/// GC runs every 60 seconds — buckets untouched for 60s are removed.
const GC_INTERVAL: Duration = Duration::from_secs(60);

/// Buckets older than this are considered stale and evicted.
const BUCKET_TTL: Duration = Duration::from_secs(120);

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// - `rps`: requests per second (tokens refilled per second)
    /// - `burst`: maximum burst size (initial tokens, and cap after refill)
    pub fn new(rps: f64, burst: f64) -> Self {
        Self {
            inner: Mutex::new(Inner {
                buckets: HashMap::new(),
                last_gc: Instant::now(),
            }),
            rps,
            burst,
        }
    }

    /// Check whether a request from `client_key` should be allowed.
    ///
    /// Returns `true` if the request passes (has a token), `false` if rate-limited.
    /// Invalid client keys (empty or containing whitespace) are mapped to "invalid"
    /// to prevent hash map poisoning attacks.
    pub async fn check(&self, client_key: &str) -> bool {
        self.check_at(client_key, Instant::now()).await
    }

    async fn check_at(&self, client_key: &str, now: Instant) -> bool {
        let mut inner = self.inner.lock().await;

        // Periodic GC
        if now.duration_since(inner.last_gc) >= GC_INTERVAL {
            inner
                .buckets
                .retain(|_, b| now.duration_since(b.last_refill) < BUCKET_TTL);
            inner.last_gc = now;
        }

        // Validate IP format (basic check: non-empty, no whitespace)
        // Invalid keys are mapped to "invalid" to share a single bucket
        let bucket_key = if client_key.is_empty() || client_key.contains(char::is_whitespace) {
            "invalid"
        } else {
            client_key
        };

        let bucket = inner
            .buckets
            .entry(bucket_key.to_string())
            .or_insert_with(|| Bucket {
                tokens: self.burst,
                last_refill: now,
            });

        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.rps).min(self.burst);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Return the number of tracked client IPs (for stats).
    pub async fn active_clients(&self) -> usize {
        let inner = self.inner.lock().await;
        inner.buckets.len()
    }

    /// Return the current drop ratio (requests being rate-limited) — placeholder.
    /// Full tracking of accepted vs rejected would require atomic counters.
    pub async fn stats_summary(&self) -> String {
        format!(
            "rps={} burst={} active_clients={}",
            self.rps,
            self.burst,
            self.active_clients().await
        )
    }
}

#[cfg(test)]
#[path = "tests/rate_limiter.rs"]
mod tests;
