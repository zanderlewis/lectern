use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

/// Shared HTTP client with optimized connection pooling and settings
/// This provides better performance for concurrent requests
pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .pool_max_idle_per_host(50) // Increase connection pool size
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .tcp_keepalive(Duration::from_secs(60))
        .http2_adaptive_window(true)
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
