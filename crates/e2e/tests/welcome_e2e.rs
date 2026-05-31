use e2e::E2eContext;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn welcome_returns_html_page() {
    let ctx = E2eContext::new().await;

    let response = ctx.server.get("/").await;

    response.assert_status_ok();
    response.assert_header("content-type", "text/html; charset=utf-8");
    response.assert_text_contains("Rust Backend Playground");
    response.assert_text_contains("/posts");
}
