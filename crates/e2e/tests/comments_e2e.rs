use e2e::E2eContext;
use serial_test::serial;

async fn create_post(ctx: &E2eContext, title: &str) -> i64 {
    let create = ctx
        .server
        .post("/posts")
        .add_header("Authorization", ctx.bearer())
        .json(&serde_json::json!({
            "title": title,
            "content": "본문"
        }))
        .await;
    create.assert_status_success();
    create.json::<serde_json::Value>()["id"]
        .as_i64()
        .expect("post id")
}

async fn create_comment(ctx: &E2eContext, post_id: i64, content: &str) -> serde_json::Value {
    let response = ctx
        .server
        .post(&format!("/posts/{post_id}/comments"))
        .add_header("Authorization", ctx.user_bearer())
        .json(&serde_json::json!({ "content": content }))
        .await;
    response.assert_status_success();
    response.json()
}

async fn create_reply(
    ctx: &E2eContext,
    post_id: i64,
    parent_comment_id: i64,
    content: &str,
) -> axum_test::TestResponse {
    ctx.server
        .post(&format!(
            "/posts/{post_id}/comments/{parent_comment_id}/replies"
        ))
        .add_header("Authorization", ctx.user_bearer())
        .json(&serde_json::json!({ "content": content }))
        .await
}

#[tokio::test]
#[serial]
async fn user_can_create_root_comment_and_replies() {
    let ctx = E2eContext::new().await;
    let post_id = create_post(&ctx, "댓글 대상").await;

    let root = create_comment(&ctx, post_id, "루트 댓글").await;
    assert_eq!(root["post_id"], post_id);
    assert_eq!(root["content"], "루트 댓글");
    assert_eq!(root["depth"], 0);

    let reply = create_reply(&ctx, post_id, root["id"].as_i64().expect("root id"), "답글").await;
    reply.assert_status_success();
    let reply: serde_json::Value = reply.json();
    assert_eq!(reply["parent_comment_id"], root["id"]);
    assert_eq!(reply["depth"], 1);
}

#[tokio::test]
#[serial]
async fn comment_depth_is_limited_to_two() {
    let ctx = E2eContext::new().await;
    let post_id = create_post(&ctx, "깊이 제한").await;

    let root = create_comment(&ctx, post_id, "루트").await;
    let reply = create_reply(
        &ctx,
        post_id,
        root["id"].as_i64().expect("root id"),
        "1단계",
    )
    .await;
    reply.assert_status_success();
    let reply: serde_json::Value = reply.json();

    let nested = create_reply(
        &ctx,
        post_id,
        reply["id"].as_i64().expect("reply id"),
        "2단계",
    )
    .await;
    nested.assert_status_success();
    let nested: serde_json::Value = nested.json();
    assert_eq!(nested["depth"], 2);

    let too_deep = create_reply(
        &ctx,
        post_id,
        nested["id"].as_i64().expect("nested id"),
        "3단계",
    )
    .await;
    too_deep.assert_status_bad_request();
}

#[tokio::test]
#[serial]
async fn list_comments_paginates_roots_and_nests_replies() {
    let ctx = E2eContext::new().await;
    let post_id = create_post(&ctx, "목록 댓글").await;

    let first = create_comment(&ctx, post_id, "첫 번째").await;
    let second = create_comment(&ctx, post_id, "두 번째").await;
    let reply = create_reply(
        &ctx,
        post_id,
        first["id"].as_i64().expect("first id"),
        "첫 답글",
    )
    .await;
    reply.assert_status_success();

    let page = ctx
        .server
        .get(&format!("/posts/{post_id}/comments?limit=1"))
        .add_header("Authorization", ctx.user_bearer())
        .await;
    page.assert_status_ok();
    let page: serde_json::Value = page.json();
    assert_eq!(page["data"].as_array().expect("comments").len(), 1);
    assert_eq!(page["data"][0]["comment"]["id"], first["id"]);
    assert_eq!(
        page["data"][0]["replies"][0]["comment"]["content"],
        "첫 답글"
    );
    assert_eq!(page["has_more"], true);

    let next_cursor = page["next_cursor"].as_i64().expect("next cursor");
    let next_page = ctx
        .server
        .get(&format!(
            "/posts/{post_id}/comments?cursor={next_cursor}&limit=1"
        ))
        .add_header("Authorization", ctx.user_bearer())
        .await;
    next_page.assert_status_ok();
    let next_page: serde_json::Value = next_page.json();
    assert_eq!(next_page["data"][0]["comment"]["id"], second["id"]);
    assert_eq!(next_page["has_more"], false);
}

#[tokio::test]
#[serial]
async fn post_detail_includes_recent_comment_threads() {
    let ctx = E2eContext::new().await;
    let post_id = create_post(&ctx, "상세 댓글").await;

    for index in 0..6 {
        create_comment(&ctx, post_id, &format!("댓글 {index}")).await;
    }

    let response = ctx
        .server
        .get(&format!("/posts/{post_id}"))
        .add_header("Authorization", ctx.user_bearer())
        .await;
    response.assert_status_ok();
    let detail: serde_json::Value = response.json();
    assert_eq!(detail["comment_count"], 6);
    assert_eq!(detail["has_more_comments"], true);
    assert_eq!(detail["comments"].as_array().expect("comments").len(), 5);
    assert_eq!(detail["comments"][0]["comment"]["content"], "댓글 5");
}

#[tokio::test]
#[serial]
async fn get_update_and_delete_own_comment() {
    let ctx = E2eContext::new().await;
    let post_id = create_post(&ctx, "수정 삭제 댓글").await;
    let comment = create_comment(&ctx, post_id, "수정 전").await;
    let comment_id = comment["id"].as_i64().expect("comment id");

    let get = ctx
        .server
        .get(&format!("/posts/{post_id}/comments/{comment_id}"))
        .add_header("Authorization", ctx.user_bearer())
        .await;
    get.assert_status_ok();
    assert_eq!(get.json::<serde_json::Value>()["content"], "수정 전");

    let update = ctx
        .server
        .put(&format!("/posts/{post_id}/comments/{comment_id}"))
        .add_header("Authorization", ctx.user_bearer())
        .json(&serde_json::json!({ "content": "수정 후" }))
        .await;
    update.assert_status_ok();
    assert_eq!(update.json::<serde_json::Value>()["content"], "수정 후");

    let delete = ctx
        .server
        .delete(&format!("/posts/{post_id}/comments/{comment_id}"))
        .add_header("Authorization", ctx.user_bearer())
        .await;
    delete.assert_status_ok();

    let get_deleted = ctx
        .server
        .get(&format!("/posts/{post_id}/comments/{comment_id}"))
        .add_header("Authorization", ctx.user_bearer())
        .await;
    get_deleted.assert_status_not_found();
}

#[tokio::test]
#[serial]
async fn rejects_other_user_writes_and_missing_or_mismatched_resources() {
    let ctx = E2eContext::new().await;
    let post_id = create_post(&ctx, "권한 댓글").await;
    let other_post_id = create_post(&ctx, "다른 글").await;
    let comment = create_comment(&ctx, post_id, "원본").await;
    let comment_id = comment["id"].as_i64().expect("comment id");

    let other_update = ctx
        .server
        .put(&format!("/posts/{post_id}/comments/{comment_id}"))
        .add_header("Authorization", ctx.bearer())
        .json(&serde_json::json!({ "content": "타인 수정" }))
        .await;
    other_update.assert_status_forbidden();

    let other_delete = ctx
        .server
        .delete(&format!("/posts/{post_id}/comments/{comment_id}"))
        .add_header("Authorization", ctx.bearer())
        .await;
    other_delete.assert_status_forbidden();

    let missing_post = ctx
        .server
        .post("/posts/999999/comments")
        .add_header("Authorization", ctx.user_bearer())
        .json(&serde_json::json!({ "content": "없음" }))
        .await;
    missing_post.assert_status_not_found();

    let missing_comment = ctx
        .server
        .get(&format!("/posts/{post_id}/comments/999999"))
        .add_header("Authorization", ctx.user_bearer())
        .await;
    missing_comment.assert_status_not_found();

    let mismatched_parent = create_reply(&ctx, other_post_id, comment_id, "다른 글 부모").await;
    mismatched_parent.assert_status_not_found();
}
