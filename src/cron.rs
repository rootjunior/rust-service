use crate::infra::clients::client::PostClient;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub struct ProjectCron;

impl ProjectCron {
    pub async fn start(
        shutdown: CancellationToken,
    ) -> Result<(), JobSchedulerError> {
        let mut scheduler = JobScheduler::new().await?;

        // 🔹 Simple cron
        scheduler
            .add(Job::new("1/10 * * * * *", |_uuid, _l| {
                info!("⏰ I run every 10 seconds");
            })?)
            .await?;

        // 🔹 Async cron
        scheduler
            .add(Job::new_async("1/1 * * * * *", |_uuid, _l| {
                Box::pin(async move {
                    info!("⏰ I run async every 7 seconds");
                    let client = PostClient {
                        url: "https://jsonplaceholder.typicode.com/posts"
                            .to_string(),
                    };
                    let result = async {
                        let posts = client.get_posts().await?;
                        for post in posts {
                            println!("📬 Post {} {}", post.id, post.body);
                        }
                        Ok::<(), reqwest::Error>(())
                    }
                    .await;

                    if let Err(err) = result {
                        error!("❌ Cron job error: {}", err);
                    }
                })
            })?)
            .await?;

        // 🔹 English cron
        scheduler
            .add(Job::new_async("every 4 seconds", |_uuid, _l| {
                Box::pin(async move {
                    info!("⏰ I run every 4 seconds");
                })
            })?)
            .await?;

        // 🔹 One-shot
        scheduler
            .add(Job::new_one_shot(Duration::from_secs(18), |_uuid, _l| {
                info!("🔥 I only run once");
            })?)
            .await?;

        // 🔹 Repeated job
        let jj = Job::new_repeated(Duration::from_secs(8), |_uuid, _l| {
            info!("🔁 I run repeatedly every 8 seconds");
        })?;
        scheduler.add(jj).await?;

        scheduler.start().await?;
        info!("✅ Cron scheduler started");

        shutdown.cancelled().await;

        info!("🛑 Cron scheduler shutting down");
        scheduler.shutdown().await?;

        Ok(())
    }
}
