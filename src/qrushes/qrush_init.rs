// test/src/qrushes/qrush_integrated.rs
use actix_web::web;
use std::sync::Arc;
use tokio::sync::{Notify, OnceCell};
use qrush::config::{QueueConfig, QUEUE_INITIALIZED, set_basic_auth, QrushBasicAuthConfig};
use qrush::registry::register_job;
use qrush::cron::cron_scheduler::CronScheduler;
use qrush::routes::metrics_route::qrush_metrics_routes;
use crate::qrushes::jobs::notify_user::NotifyUser;
use crate::qrushes::jobs::dispatch_ride_job::DispatchRideJob;
use crate::qrushes::crons::daily_report_job::DailyReportJob;
use nanoid::nanoid;

// Integrated-specific initialization tracker
static QRUSH_INTEGRATION_INIT: OnceCell<Arc<Notify>> = OnceCell::const_new();

pub struct QrushInit;

#[derive(Clone, Debug)]
pub struct QrushWorkerConfig {
    pub worker_id: String,
    pub initialized_at: std::time::SystemTime,
    pub integration_mode: String,
}

impl QrushInit {
    /// 🌍 GLOBAL initialization - call this ONCE in main.rs
    pub async fn initialize(basic_auth: Option<QrushBasicAuthConfig>) {
        // Check if already initialized globally
        if let Some(existing_notify) = QRUSH_INTEGRATION_INIT.get() {
            println!("QRush already initialized globally (integrated mode), waiting for completion...");
            existing_notify.notified().await;
            return;
        }

        let queue_notify = Arc::new(Notify::new());
        let _ = QRUSH_INTEGRATION_INIT.set(queue_notify.clone());

        println!("🌍 Starting GLOBAL QRush initialization (INTEGRATED mode)...");

        let basic_auth = basic_auth.or_else(|| {
            std::env::var("QRUSH_BASIC_AUTH").ok().and_then(|auth| {
                let parts: Vec<&str> = auth.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some(QrushBasicAuthConfig {
                        username: parts[0].to_string(),
                        password: parts[1].to_string(),
                    })
                } else {
                    None
                }
            })
        });

        let _ = set_basic_auth(basic_auth);
        let _ = QUEUE_INITIALIZED.set(queue_notify.clone());

        // Register jobs globally
        println!("Registering jobs for integrated mode...");
        register_job(NotifyUser::name(), NotifyUser::handler);
        register_job(DispatchRideJob::name(), DispatchRideJob::handler);
        register_job(DailyReportJob::name(), DailyReportJob::handler);

        // Initialize queues in background
        tokio::spawn({
            let queue_notify = queue_notify.clone();
            async move {
                let redis_url = std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

                println!("Connecting to Redis: {}", redis_url);

                let queues = vec![
                    QueueConfig::new("dispatch", 5, 1),
                    QueueConfig::new("default", 5, 1),
                    QueueConfig::new("critical", 10, 0),
                ];

                if let Err(err) = QueueConfig::initialize(redis_url, queues).await {
                    eprintln!("Failed to initialize qrush (integrated): {:?}", err);
                } else {
                    println!("QRush queues started successfully (integrated mode)");
                }
                
                queue_notify.notify_waiters();
            }
        });
        // Wait for queue initialization
        queue_notify.notified().await;
        println!("🚀 Global queue initialization complete (integrated mode)");
        // Register cron jobs after queues are ready
        Self::register_cron_jobs().await;
        println!("🎯 GLOBAL QRush initialization complete (INTEGRATED mode)!");
    }

    /// Register cron jobs for integrated mode
    async fn register_cron_jobs() {
        println!("Registering integrated mode cron jobs...");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let daily_report_job = DailyReportJob {
            report_type: "integrated_report".to_string(),
        };
        
        match CronScheduler::register_cron_job(daily_report_job).await {
            Ok(_) => {
                println!("DailyReportJob Cron Job registered for integrated mode");
            }
            Err(e) => {
                println!("Failed to register integrated Cron Job: {:?}", e);
            }
        }
    }


    /*-----------------------------------------------------------
    Utilities
    ------------------------------------------------------------*/

    // Generate nano uniq id
    pub fn gen_uniq_nanoid() -> String {
        nanoid!()
    }

    // WORKER setup - call this in each HttpServer::new()
    // used for debugging/monitoring purposes
    // fn test(qrush_worker_config: web::Data<QrushWorkerConfig>)
    pub fn setup_worker_sync() -> QrushWorkerConfig {
        let worker_id = Self::gen_uniq_nanoid();
        println!("Setting up QRush integrated worker: {}", worker_id);
        QrushWorkerConfig {
            worker_id,
            initialized_at: std::time::SystemTime::now(),
            integration_mode: "integrated".to_string(),
        }
    }

    /// Get QRush metrics routes for integration into main app
    pub fn configure_routes(cfg: &mut web::ServiceConfig) {
        println!("🔧 Configuring integrated QRush routes...");
        qrush_metrics_routes(cfg);
    }

    /// Check if QRush is initialized
    pub fn is_initialized() -> bool {
        QRUSH_INTEGRATION_INIT.get().is_some()
    }
    /*-----------------------------------------------------------
    Utilities
    ------------------------------------------------------------*/
}
