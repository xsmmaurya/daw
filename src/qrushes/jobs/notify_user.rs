// src/qrushes/jobs/notify_user.rs
use async_trait::async_trait;
use futures::future::BoxFuture;
use qrush::job::Job;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NotifyUser {
    pub user_id: String,
    pub message: String,
}

#[async_trait]
impl Job for NotifyUser {
    async fn perform(&self) -> Result<()> {
        // For now just log; you can wire email/SMS later
        println!("Notify user {} -> {}", self.user_id, self.message);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "NotifyUser"
    }

    fn queue(&self) -> &'static str {
        "default"
    }
}

impl NotifyUser {
    pub fn name() -> &'static str {
        "NotifyUser"
    }

    pub fn handler(payload: String) -> BoxFuture<'static, Result<Box<dyn Job>>> {
        Box::pin(async move {
            let job: NotifyUser = serde_json::from_str(&payload)?;
            Ok(Box::new(job) as Box<dyn Job>)
        })
    }
}
