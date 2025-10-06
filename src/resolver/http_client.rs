use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

/// Shared HTTP client with optimized connection pooling and settings
/// This provides better performance for concurrent requests
pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .pool_max_idle_per_host(100) // Increase connection pool size for better concurrency
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(5)) // Faster connection timeout
        .tcp_keepalive(Duration::from_secs(60))
        .tcp_nodelay(true) // Disable Nagle's algorithm for lower latency
        .http2_adaptive_window(true) // Use HTTP/2 when available, fallback to HTTP/1.1
        .http2_keep_alive_interval(Duration::from_secs(30))
        .http2_keep_alive_timeout(Duration::from_secs(20))
        .http2_keep_alive_while_idle(true)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .user_agent("Lectern/0.1.0 (Rust; async)")
        .build()
        .expect("Failed to build HTTP client")
});

/// Get the shared HTTP client instance
pub fn get_client() -> &'static Client {
    &HTTP_CLIENT
}
