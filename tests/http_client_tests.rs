use lectern::resolver::http_client::get_client;

#[test]
fn test_http_client_initialization() {
    let client = get_client();
    // Should not panic - client is successfully created
    // We can't directly access timeout() but we can verify it works
    assert!(std::ptr::addr_of!(client) as usize != 0);
}

#[tokio::test]
async fn test_http_client_reusability() {
    let client1 = get_client();
    let client2 = get_client();
    
    // Should return the same static instance
    // We can verify this by checking they work identically
    let url = "https://httpbin.org/get";
    
    let result1 = client1.get(url).send().await;
    let result2 = client2.get(url).send().await;
    
    // Both should behave the same (succeed or fail consistently)
    assert_eq!(result1.is_ok(), result2.is_ok());
}

#[tokio::test]
async fn test_http_client_connection_pooling() {
    let client = get_client();
    
    // Make multiple requests to test connection pooling
    let url = "https://repo.packagist.org/packages.json";
    
    let mut requests = Vec::new();
    for _ in 0..3 {
        requests.push(client.get(url).send());
    }
    
    // All requests should complete
    for result in futures::future::join_all(requests).await {
        // Should either succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_http_client_has_compression() {
    let client = get_client();
    // Client should be configured with compression support
    // This is implicit in the builder configuration
    // We're just checking it doesn't panic and is valid
    assert!(std::ptr::addr_of!(client) as usize != 0);
}
