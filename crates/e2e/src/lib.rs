use std::sync::Arc;

use api::http::AppState;
use api::http::routes::configure_routes;
use application::password;
use application::post::service::PostService;
use application::user::{AuthService, JwtConfig, UserService};
use axum_test::TestServer;
use chrono::Utc;
use infrastructure::cache::post_cache::NoopPostCache;
use infrastructure::persistence::seaorm::post::Entity as PostEntity;
use infrastructure::persistence::seaorm::post_repository::{PostRepository, SeaOrmPostRepository};
use infrastructure::persistence::seaorm::post_revision_repository::{
    PostRevisionRepository, SeaOrmPostRevisionRepository,
};
use infrastructure::persistence::seaorm::user::{
    ActiveModel as UserActiveModel, Entity as UserEntity,
};
use infrastructure::persistence::seaorm::user_repository::{SeaOrmUserRepository, UserRepository};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::ImageExt;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use tokio::sync::OnceCell;

mod token_mock;

use token_mock::test_token_repository;

static POSTGRES: OnceCell<(ContainerAsync<Postgres>, String)> = OnceCell::const_new();

const TEST_JWT_SECRET: &str = "e2e-test-jwt-secret-key-min-32-chars";
const EDITOR_USER_ID: &str = "e2e-editor-user";
const EDITOR_USERNAME: &str = "editor";
const EDITOR_PASSWORD: &str = "password";
const REGULAR_USER_ID: &str = "e2e-regular-user";
const REGULAR_USERNAME: &str = "regular";
const REGULAR_PASSWORD: &str = "password";

/// PostgreSQL(Testcontainers) + Axum 라우터를 묶은 E2E 테스트 컨텍스트.
pub struct E2eContext {
    pub server: TestServer,
    access_token: String,
    user_access_token: String,
}

impl E2eContext {
    pub async fn new() -> Self {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init();
        dotenvy::dotenv().ok();

        let database_url = match std::env::var("E2E_DATABASE_URL") {
            Ok(url) => url,
            Err(_) => testcontainer_database_url().await,
        };

        let db = infrastructure::db::connect(&database_url)
            .await
            .expect("PostgreSQL에 연결할 수 없습니다. Docker가 실행 중인지 확인하세요.");

        clean_data(&db).await;
        seed_editor_user(&db).await;
        seed_regular_user(&db).await;

        let jwt_config = JwtConfig {
            secret: TEST_JWT_SECRET.to_string(),
            access_expiry_secs: 900,
            refresh_expiry_secs: 604_800,
        };

        let post_repo: Arc<dyn PostRepository> = Arc::new(SeaOrmPostRepository::new(db.clone()));
        let revision_repo: Arc<dyn PostRevisionRepository> =
            Arc::new(SeaOrmPostRevisionRepository::new(db.clone()));
        let user_repo: Arc<dyn UserRepository> = Arc::new(SeaOrmUserRepository::new(db.clone()));
        let token_repo = test_token_repository();
        let post_cache = Arc::new(NoopPostCache);

        let post_service = Arc::new(PostService::new(
            post_repo.clone(),
            post_cache,
            Some(revision_repo),
        ));
        let auth_service = Arc::new(AuthService::new(
            user_repo.clone(),
            token_repo,
            jwt_config.clone(),
        ));
        let user_service = Arc::new(UserService::new(user_repo.clone()));

        let access_token = auth_service
            .login(application::user::LoginCmd {
                username: EDITOR_USERNAME.to_string(),
                password: EDITOR_PASSWORD.to_string(),
            })
            .await
            .expect("editor login")
            .access_token;

        let user_access_token = auth_service
            .login(application::user::LoginCmd {
                username: REGULAR_USERNAME.to_string(),
                password: REGULAR_PASSWORD.to_string(),
            })
            .await
            .expect("regular login")
            .access_token;

        let notification_service = Arc::new(
            application::notification::service::NotificationService::new(
                Arc::new(
                    application::notification::seaorm_repository::SeaOrmNotificationRepository::new(
                        db.clone(),
                    ),
                ),
                None,
            ),
        );
        let comment_repo: Arc<
            dyn infrastructure::persistence::seaorm::comment_repository::CommentRepository,
        > = Arc::new(
            infrastructure::persistence::seaorm::comment_repository::SeaOrmCommentRepository::new(
                db.clone(),
            ),
        );
        let comment_service = Arc::new(application::comment::service::CommentService::new(
            comment_repo,
            post_repo,
            Some(notification_service.clone()),
        ));

        let app_state = AppState {
            post_service,
            comment_service,
            tag_service: Arc::new(application::tag::service::TagService::new(Arc::new(
                infrastructure::persistence::seaorm::tag_repository::SeaOrmTagRepository::new(
                    db.clone(),
                ),
            ))),
            auth_service,
            user_service,
            jwt_config: Arc::new(jwt_config),
            notification_service,
            like_service: Arc::new(application::like::service::LikeService::new(Arc::new(
                infrastructure::persistence::seaorm::like_repository::SeaOrmLikeRepository::new(db),
            ))),
            rate_limiter: None,
        };
        let app = configure_routes(app_state);
        let server = TestServer::new(app).expect("TestServer 생성 실패");

        Self {
            server,
            access_token,
            user_access_token,
        }
    }

    pub fn bearer(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    pub fn user_bearer(&self) -> String {
        format!("Bearer {}", self.user_access_token)
    }
}

async fn testcontainer_database_url() -> String {
    POSTGRES
        .get_or_init(|| async {
            let container = Postgres::default()
                .with_db_name("playground")
                .with_user("postgres")
                .with_password("postgrespassword")
                .with_tag("15-alpine")
                .start()
                .await
                .expect("PostgreSQL Testcontainer 시작 실패. Docker 데몬이 필요합니다.");

            let host = container.get_host().await.expect("Testcontainer host");
            let port = container
                .get_host_port_ipv4(5432)
                .await
                .expect("Testcontainer port");

            let url = format!("postgres://postgres:postgrespassword@{host}:{port}/playground");
            (container, url)
        })
        .await
        .1
        .clone()
}

async fn clean_data(db: &DatabaseConnection) {
    PostEntity::delete_many()
        .exec(db)
        .await
        .expect("posts 테이블 정리 실패");
    UserEntity::delete_many()
        .exec(db)
        .await
        .expect("users 테이블 정리 실패");
}

async fn seed_editor_user(db: &DatabaseConnection) {
    let password_hash = password::hash_password(EDITOR_PASSWORD).expect("password hash");
    let now = Utc::now();

    let user = UserActiveModel {
        id: Set(EDITOR_USER_ID.to_string()),
        username: Set(EDITOR_USERNAME.to_string()),
        email: Set("editor@example.com".to_string()),
        password_hash: Set(password_hash),
        role: Set("editor".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        bio: Set(None),
        avatar_url: Set(None),
        last_login_at: Set(None),
    };

    user.insert(db).await.expect("editor 사용자 시드 실패");
}

async fn seed_regular_user(db: &DatabaseConnection) {
    let password_hash = password::hash_password(REGULAR_PASSWORD).expect("password hash");
    let now = Utc::now();

    let user = UserActiveModel {
        id: Set(REGULAR_USER_ID.to_string()),
        username: Set(REGULAR_USERNAME.to_string()),
        email: Set("regular@example.com".to_string()),
        password_hash: Set(password_hash),
        role: Set("user".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        bio: Set(None),
        avatar_url: Set(None),
        last_login_at: Set(None),
    };

    user.insert(db).await.expect("regular 사용자 시드 실패");
}
