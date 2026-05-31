use std::sync::Arc;

use application::post::batch::PostBatchService;
use infrastructure::cache::valkey::ValkeyCache;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub struct BatchContext {
    pub post_batch_service: Arc<PostBatchService>,
    pub valkey_cache: Arc<Mutex<Option<ValkeyCache>>>,
}

impl BatchContext {
    pub async fn new(post_batch_service: Arc<PostBatchService>, valkey_url: Option<&str>) -> Self {
        let valkey_cache = match valkey_url {
            Some(url) => {
                info!("Connecting to Valkey at {}...", url);
                match ValkeyCache::connect(url).await {
                    Ok(cache) => {
                        info!("Valkey connected.");
                        Some(cache)
                    }
                    Err(e) => {
                        warn!(
                            "Valkey connection failed (count sync to cache disabled): {}",
                            e
                        );
                        None
                    }
                }
            }
            None => {
                info!("VALKEY_URL not set; post count will be logged only.");
                None
            }
        };

        Self {
            post_batch_service,
            valkey_cache: Arc::new(Mutex::new(valkey_cache)),
        }
    }
}
