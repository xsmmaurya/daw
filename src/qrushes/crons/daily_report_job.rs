// src/qrushes/crons/daily_report_job.rs
use async_trait::async_trait;
use futures::future::BoxFuture;
use qrush::job::Job;
use qrush::cron::cron_job::CronJob;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyReportJob {
    pub report_type: String,
}

#[async_trait]
impl Job for DailyReportJob {
    async fn perform(&self) -> Result<()> {
        // Placeholder: you can generate your own reports here
        println!("Generating daily report: {}", self.report_type);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "DailyReportJob"
    }

    fn queue(&self) -> &'static str {
        "default"
    }
}

#[async_trait]
impl CronJob for DailyReportJob {
    fn cron_expression(&self) -> &'static str {
        // Every minute (easy for local testing)
        "0 * * * * *"
    }

    fn cron_id(&self) -> &'static str {
        "daily_report"
    }
}

impl DailyReportJob {
    pub fn name() -> &'static str {
        "DailyReportJob"
    }

    pub fn handler(payload: String) -> BoxFuture<'static, Result<Box<dyn Job>>> {
        Box::pin(async move {
            let job: DailyReportJob = serde_json::from_str(&payload)?;
            Ok(Box::new(job) as Box<dyn Job>)
        })
    }
}
