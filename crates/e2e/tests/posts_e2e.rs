use e2e::E2eContext;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn create_list_and_get_post() {
    let ctx = E2eContext::new().await;

    let create = ctx
        .server
        .post("/posts")
        .json(&serde_json::json!({
            "title": "E2E 포스트",
            "content": "본문 내용"
        }))
        .await;
    create.assert_status_ok();

    let created: serde_json::Value = create.json();
    assert_eq!(created["title"], "E2E 포스트");
    assert_eq!(created["content"], "본문 내용");
    assert_eq!(created["status"], "draft");
    let id = created["id"].as_i64().expect("post id");

    let list = ctx.server.get("/posts").await;
    list.assert_status_ok();
    let posts: Vec<serde_json::Value> = list.json();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0]["id"], id);

    let get = ctx.server.get(&format!("/posts/{id}")).await;
    get.assert_status_ok();
    let fetched: serde_json::Value = get.json();
    assert_eq!(fetched["title"], "E2E 포스트");
}

#[tokio::test]
#[serial]
async fn update_post() {
    let ctx = E2eContext::new().await;

    let create = ctx
        .server
        .post("/posts")
        .json(&serde_json::json!({
            "title": "수정 전",
            "content": "원본"
        }))
        .await;
    create.assert_status_ok();
    let id = create.json::<serde_json::Value>()["id"]
        .as_i64()
        .expect("post id");

    let update = ctx
        .server
        .put(&format!("/posts/{id}"))
        .json(&serde_json::json!({
            "title": "수정 후",
            "content": "변경됨"
        }))
        .await;
    update.assert_status_ok();

    let updated: serde_json::Value = update.json();
    assert_eq!(updated["title"], "수정 후");
    assert_eq!(updated["content"], "변경됨");
    assert_eq!(updated["status"], "draft");
}

#[tokio::test]
#[serial]
async fn publish_post() {
    let ctx = E2eContext::new().await;

    let create = ctx
        .server
        .post("/posts")
        .json(&serde_json::json!({
            "title": "발행 대상",
            "content": "초안"
        }))
        .await;
    create.assert_status_ok();
    let id = create.json::<serde_json::Value>()["id"]
        .as_i64()
        .expect("post id");

    let publish = ctx.server.post(&format!("/posts/{id}/publish")).await;
    publish.assert_status_ok();

    let published: serde_json::Value = publish.json();
    assert_eq!(published["status"], "published");

    let republish = ctx.server.post(&format!("/posts/{id}/publish")).await;
    republish.assert_status_bad_request();
}

#[tokio::test]
#[serial]
async fn delete_post() {
    let ctx = E2eContext::new().await;

    let create = ctx
        .server
        .post("/posts")
        .json(&serde_json::json!({
            "title": "삭제 대상",
            "content": "임시"
        }))
        .await;
    create.assert_status_ok();
    let id = create.json::<serde_json::Value>()["id"]
        .as_i64()
        .expect("post id");

    let delete = ctx.server.delete(&format!("/posts/{id}")).await;
    delete.assert_status_ok();

    let body: serde_json::Value = delete.json();
    assert_eq!(body["success"], true);

    let get = ctx.server.get(&format!("/posts/{id}")).await;
    get.assert_status_not_found();
}

#[tokio::test]
#[serial]
async fn create_post_rejects_empty_title() {
    let ctx = E2eContext::new().await;

    let response = ctx
        .server
        .post("/posts")
        .json(&serde_json::json!({
            "title": "   ",
            "content": "본문"
        }))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
#[serial]
async fn get_nonexistent_post_returns_not_found() {
    let ctx = E2eContext::new().await;

    let response = ctx.server.get("/posts/999999").await;

    response.assert_status_not_found();
}
