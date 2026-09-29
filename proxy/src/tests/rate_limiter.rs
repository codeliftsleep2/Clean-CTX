use super::*;

#[tokio::test]
async fn test_burst_allows_initial_requests() {
    let limiter = RateLimiter::new(10.0, 5.0);
    let now = Instant::now();

    // A full bucket allows exactly the configured burst at one instant.
    for _ in 0..5 {
        assert!(limiter.check_at("127.0.0.1", now).await);
    }
    assert!(!limiter.check_at("127.0.0.1", now).await);
}

#[tokio::test]
async fn test_refills_over_time() {
    let limiter = RateLimiter::new(10.0, 5.0);
    let start = Instant::now();

    for _ in 0..5 {
        assert!(limiter.check_at("127.0.0.1", start).await);
    }
    assert!(!limiter.check_at("127.0.0.1", start).await);

    // At 10 rps, advancing by 250 ms adds 2.5 tokens: two whole requests.
    let after_refill = start + Duration::from_millis(250);
    assert!(limiter.check_at("127.0.0.1", after_refill).await);
    assert!(limiter.check_at("127.0.0.1", after_refill).await);
    assert!(!limiter.check_at("127.0.0.1", after_refill).await);
}

#[tokio::test]
async fn test_different_ips_independent() {
    let limiter = RateLimiter::new(10.0, 3.0);
    let now = Instant::now();

    assert!(limiter.check_at("10.0.0.1", now).await);
    assert!(limiter.check_at("10.0.0.2", now).await);
    assert!(limiter.check_at("10.0.0.1", now).await);
    assert!(limiter.check_at("10.0.0.2", now).await);
}

#[tokio::test]
async fn test_gc_culls_stale_entries() {
    let limiter = RateLimiter::new(10.0, 5.0);
    limiter.check("stale-client").await;
    assert_eq!(limiter.active_clients().await, 1);
    // This verifies active-client accounting without waiting for the GC interval.
}

#[tokio::test]
async fn test_no_tokens_no_burst() {
    let limiter = RateLimiter::new(0.0, 0.0);
    let now = Instant::now();

    assert!(!limiter.check_at("127.0.0.1", now).await);
    assert!(!limiter.check_at("127.0.0.1", now).await);
}

#[tokio::test]
async fn test_high_rps_with_low_burst() {
    let limiter = RateLimiter::new(100.0, 2.0);
    let start = Instant::now();

    assert!(limiter.check_at("127.0.0.1", start).await);
    assert!(limiter.check_at("127.0.0.1", start).await);
    assert!(!limiter.check_at("127.0.0.1", start).await);

    // Advance just beyond the floating-point boundary for one refilled token.
    assert!(
        limiter
            .check_at("127.0.0.1", start + Duration::from_millis(11))
            .await
    );
}

#[tokio::test]
async fn test_invalid_ip_validation() {
    let limiter = RateLimiter::new(10.0, 5.0);
    let now = Instant::now();

    // Empty and whitespace-only keys share the single "invalid" bucket.
    assert!(limiter.check_at("", now).await);
    assert!(limiter.check_at("   ", now).await);
    assert!(limiter.check_at("", now).await);
    assert!(limiter.check_at("", now).await);
    assert!(limiter.check_at("", now).await);
    assert!(!limiter.check_at("", now).await);
    assert!(!limiter.check_at("   ", now).await);
}
