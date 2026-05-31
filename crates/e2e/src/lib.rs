use std::sync::Arc;

use api::http::AppState;
use api::http::routes::configure_routes;
use application::post::service::PostService;
use axum_test::TestServer;
use infrastructure::persistence::seaorm::post::Entity as PostEntity;
use infrastructure::persistence::seaorm::post_repository::{PostRepository, SeaOrmPostRepository};
use sea_orm::{DatabaseConnection, EntityTrait};
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ContainerAsync;
use tokio::sync::OnceCell;

static POSTGRES: OnceCell<(ContainerAsync<Postgres>, String)> = OnceCell::const_new();

/// PostgreSQL(Testcontainers) + Axum 라우터를 묶은 E2E 테스트 컨텍스트.
pub struct E2eContext {
    pub server: TestServer,
}

impl E2eContext {
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();

        let database_url = match std::env::var("E2E_DATABASE_URL") {
            Ok(url) => url,
            Err(_) => testcontainer_database_url().await,
        };

        let db = infrastructure::db::connect(&database_url)
            .await
            .expect("PostgreSQL에 연결할 수 없습니다. Docker가 실행 중인지 확인하세요.");

        clean_posts(&db).await;

        let post_repo: Arc<dyn PostRepository> = Arc::new(SeaOrmPostRepository::new(db));
        let post_service = Arc::new(PostService::new(post_repo));
        let app_state = AppState { post_service };
        let app = configure_routes(app_state);
        let server = TestServer::new(app).expect("TestServer 생성 실패");

        Self { server }
    }
}

async fn testcontainer_database_url() -> String {
    POSTGRES
        .get_or_init(|| async {
            let container = Postgres::default()
                .with_db_name("playground")
                .with_user("postgres")
                .with_password("postgrespassword")
                .start()
                .await
                .expect("PostgreSQL Testcontainer 시작 실패. Docker 데몬이 필요합니다.");

            let host = container.get_host().await.expect("Testcontainer host");
            let port = container
                .get_host_port_ipv4(5432)
                .await
                .expect("Testcontainer port");

            let url = format!(
                "postgres://postgres:postgrespassword@{host}:{port}/playground"
            );
            (container, url)
        })
        .await
        .1
        .clone()
}

async fn clean_posts(db: &DatabaseConnection) {
    PostEntity::delete_many()
        .exec(db)
        .await
        .expect("posts 테이블 정리 실패");
}
