use tokio::signal;
use tokio_cron_scheduler::{JobScheduler, JobSchedulerError};
use tracing::info;

pub struct Scheduler {
    inner: JobScheduler,
}

impl Scheduler {
    pub async fn new() -> Result<Self, JobSchedulerError> {
        Ok(Self {
            inner: JobScheduler::new().await?,
        })
    }

    pub async fn add(&self, job: tokio_cron_scheduler::Job) -> Result<(), JobSchedulerError> {
        self.inner.add(job).await?;
        Ok(())
    }

    pub async fn run_until_shutdown(mut self) -> Result<(), JobSchedulerError> {
        self.inner.start().await?;

        signal::ctrl_c()
            .await
            .expect("failed to listen for shutdown signal");
        info!("Shutdown signal received, stopping batch worker.");

        self.inner.shutdown().await
    }
}
