use std::sync::Arc;

use infrastructure::persistence::seaorm::post_repository::PostRepository;

pub struct PostBatchService {
    repo: Arc<dyn PostRepository>,
}

impl PostBatchService {
    pub fn new(repo: Arc<dyn PostRepository>) -> Self {
        Self { repo }
    }

    /// DB에서 포스트 개수를 조회합니다 (배치 스냅샷용).
    pub async fn snapshot_post_count(&self) -> Result<u64, String> {
        self.repo.count().await
    }
}
