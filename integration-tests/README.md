# Integration tests

This crate will use isok components as external crates, and run tests 
through them.

## Debug a test

You can add logging to your test to debug it by enabling `tracing_subscriber`:

```rust
#[tokio::test]
async fn test_agent_feedback_ok() {
    tracing_subscriber::fmt::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
    // Your test code here
}
```