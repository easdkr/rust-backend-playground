use std::sync::Arc;

use tokio_cron_scheduler::{Job, JobSchedulerError};
use tracing::{error, info};

use crate::context::BatchContext;
use crate::scheduler::Scheduler;

/// 매 분 0초에 실행 (UTC).
const SCHEDULE: &str = "0 */1 * * * *";

pub async fn register(
    scheduler: &Scheduler,
    ctx: Arc<BatchContext>,
) -> Result<(), JobSchedulerError> {
    scheduler
        .add(Job::new_async(SCHEDULE, move |_uuid, _lock| {
            let ctx = Arc::clone(&ctx);
            Box::pin(async move {
                if let Err(e) = run(&ctx).await {
                    error!("post_count job failed: {}", e);
                }
            })
        })?)
        .await?;

    info!("Registered post_count job (schedule: {})", SCHEDULE);
    Ok(())
}

async fn run(ctx: &BatchContext) -> Result<(), String> {
    let count = ctx.post_batch_service.snapshot_post_count().await?;

    let mut cache_guard = ctx.valkey_cache.lock().await;
    if let Some(cache) = cache_guard.as_mut() {
        cache.set_post_count(count).await?;
        info!("Post count snapshot synced to Valkey: {}", count);
    } else {
        info!("Post count snapshot: {}", count);
    }

    Ok(())
}
