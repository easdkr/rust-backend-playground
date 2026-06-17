use e2e::E2eContext;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn create_list_and_get_post() {
    let ctx = E2eContext::new().await;

    let create = ctx
        .server
        .post("/posts")
        .add_header("Authorization", ctx.bearer())
        .json(&serde_json::json!({
            "title": "E2E 포스트",
            "content": "본문 내용"
        }))
        .await;
    create.assert_status_success();

    let created: serde_json::Value = create.json();
    assert_eq!(created["title"], "E2E 포스트");
    assert_eq!(created["content"], "본문 내용");
    assert_eq!(created["status"], "draft");
    let id = created["id"].as_i64().expect("post id");

    let list = ctx
        .server
        .get("/posts")
        .add_header("Authorization", ctx.bearer())
        .await;
    list.assert_status_ok();
    let posts: serde_json::Value = list.json();
    assert_eq!(posts["data"].as_array().expect("posts").len(), 1);
    assert_eq!(posts["data"][0]["id"], id);

    let get = ctx
        .server
        .get(&format!("/posts/{id}"))
        .add_header("Authorization", ctx.bearer())
        .await;
    get.assert_status_ok();
    let fetched: serde_json::Value = get.json();
    assert_eq!(fetched["title"], "E2E 포스트");
    assert_eq!(fetched["comment_count"], 0);
    assert_eq!(fetched["has_more_comments"], false);
}

#[tokio::test]
#[serial]
async fn update_post() {
    let ctx = E2eContext::new().await;

    let create = ctx
        .server
        .post("/posts")
        .add_header("Authorization", ctx.bearer())
        .json(&serde_json::json!({
            "title": "수정 전",
            "content": "원본"
        }))
        .await;
    create.assert_status_success();
    let id = create.json::<serde_json::Value>()["id"]
        .as_i64()
        .expect("post id");

    let update = ctx
        .server
        .put(&format!("/posts/{id}"))
        .add_header("Authorization", ctx.bearer())
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
        .add_header("Authorization", ctx.bearer())
        .json(&serde_json::json!({
            "title": "발행 대상",
            "content": "초안"
        }))
        .await;
    create.assert_status_success();
    let id = create.json::<serde_json::Value>()["id"]
        .as_i64()
        .expect("post id");

    let publish = ctx
        .server
        .post(&format!("/posts/{id}/publish"))
        .add_header("Authorization", ctx.bearer())
        .await;
    publish.assert_status_ok();

    let published: serde_json::Value = publish.json();
    assert_eq!(published["status"], "published");

    let republish = ctx
        .server
        .post(&format!("/posts/{id}/publish"))
        .add_header("Authorization", ctx.bearer())
        .await;
    republish.assert_status_bad_request();
}

#[tokio::test]
#[serial]
async fn delete_post() {
    let ctx = E2eContext::new().await;

    let create = ctx
        .server
        .post("/posts")
        .add_header("Authorization", ctx.bearer())
        .json(&serde_json::json!({
            "title": "삭제 대상",
            "content": "임시"
        }))
        .await;
    create.assert_status_success();
    let id = create.json::<serde_json::Value>()["id"]
        .as_i64()
        .expect("post id");

    let delete = ctx
        .server
        .delete(&format!("/posts/{id}"))
        .add_header("Authorization", ctx.bearer())
        .await;
    delete.assert_status_ok();

    let body: serde_json::Value = delete.json();
    assert_eq!(body["success"], true);

    let get = ctx
        .server
        .get(&format!("/posts/{id}"))
        .add_header("Authorization", ctx.bearer())
        .await;
    get.assert_status_not_found();
}

#[tokio::test]
#[serial]
async fn create_post_rejects_empty_title() {
    let ctx = E2eContext::new().await;

    let response = ctx
        .server
        .post("/posts")
        .add_header("Authorization", ctx.bearer())
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

    let response = ctx
        .server
        .get("/posts/999999")
        .add_header("Authorization", ctx.bearer())
        .await;

    response.assert_status_not_found();
}

#[tokio::test]
#[serial]
async fn update_post_creates_revision_and_restore_works() {
    let ctx = E2eContext::new().await;

    let create = ctx
        .server
        .post("/posts")
        .add_header("Authorization", ctx.bearer())
        .json(&serde_json::json!({
            "title": "revision 원본",
            "content": "원본 내용"
        }))
        .await;
    create.assert_status_success();
    let id = create.json::<serde_json::Value>()["id"]
        .as_i64()
        .expect("post id");

    let update = ctx
        .server
        .put(&format!("/posts/{id}"))
        .add_header("Authorization", ctx.bearer())
        .json(&serde_json::json!({
            "title": "revision 수정",
            "content": "수정 내용"
        }))
        .await;
    update.assert_status_ok();

    let revisions = ctx
        .server
        .get(&format!("/posts/{id}/revisions"))
        .add_header("Authorization", ctx.bearer())
        .await;
    revisions.assert_status_ok();
    let body: serde_json::Value = revisions.json();
    let list = body.as_array().expect("revisions");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["title"], "revision 원본");
    assert_eq!(list[0]["content"], "원본 내용");
    assert_eq!(list[0]["version"], 1);

    let version = list[0]["version"].as_i64().expect("version");
    let restore = ctx
        .server
        .post(&format!("/posts/{id}/revisions/{version}/restore"))
        .add_header("Authorization", ctx.bearer())
        .await;
    restore.assert_status_ok();
    let restored: serde_json::Value = restore.json();
    assert_eq!(restored["title"], "revision 원본");
    assert_eq!(restored["content"], "원본 내용");

    let revisions2 = ctx
        .server
        .get(&format!("/posts/{id}/revisions"))
        .add_header("Authorization", ctx.bearer())
        .await;
    revisions2.assert_status_ok();
    assert_eq!(
        revisions2
            .json::<serde_json::Value>()
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
#[serial]
async fn search_posts_with_full_text_search() {
    let ctx = E2eContext::new().await;

    let create = ctx
        .server
        .post("/posts")
        .add_header("Authorization", ctx.bearer())
        .json(&serde_json::json!({
            "title": "검색 대상 제목",
            "content": "검색 대상 본문"
        }))
        .await;
    create.assert_status_success();

    let search = ctx
        .server
        .get("/posts/search?q=검색")
        .add_header("Authorization", ctx.bearer())
        .await;
    search.assert_status_ok();
    let body: serde_json::Value = search.json();
    assert_eq!(body["data"].as_array().expect("results").len(), 1);
    assert_eq!(body["data"][0]["title"], "검색 대상 제목");
}
