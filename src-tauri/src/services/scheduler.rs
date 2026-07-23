// services/scheduler.rs
//
// Cron-based macro scheduler built on top of tokio-cron-scheduler.
// Each `MacroTrigger::Cron` from a saved macro is registered here.
// When the cron fires, the scheduler enqueues macro execution through
// the existing `agent.rs` `MacroAgent`.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

use crate::error::CntrlError;

/// A handle to an active scheduled macro job.
#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub macro_id: String,
    pub cron: String,
    pub job_uuid: Uuid,
}

/// Manages cron-triggered macro executions.
/// One `MacroScheduler` is managed as Tauri state.
pub struct MacroScheduler {
    sched: Arc<JobScheduler>,
    jobs: Arc<Mutex<HashMap<String, ScheduledJob>>>,
}

impl MacroScheduler {
    /// Create and start the underlying cron scheduler.
    pub async fn new() -> Result<Self, CntrlError> {
        let sched = JobScheduler::new()
            .await
            .map_err(|e| CntrlError::Macro(format!("Failed to create scheduler: {e}")))?;

        sched
            .start()
            .await
            .map_err(|e| CntrlError::Macro(format!("Failed to start scheduler: {e}")))?;

        Ok(Self {
            sched: Arc::new(sched),
            jobs: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Schedule a macro to run on the given cron expression.
    /// `on_fire` is called each time the cron triggers.
    /// Returns the scheduled job UUID (can be used to remove it later).
    pub async fn schedule(
        &self,
        macro_id: String,
        cron_expr: &str,
        on_fire: impl Fn(String) + Send + Sync + 'static,
    ) -> Result<Uuid, CntrlError> {
        let macro_id_clone = macro_id.clone();
        let cron_clone = cron_expr.to_string();

        let job = Job::new(cron_clone.as_str(), move |_uuid, _l| {
            on_fire(macro_id_clone.clone());
        })
        .map_err(|e| CntrlError::Macro(format!("Invalid cron expression '{cron_expr}': {e}")))?;

        let job_uuid = job.guid();

        self.sched
            .add(job)
            .await
            .map_err(|e| CntrlError::Macro(format!("Failed to add job: {e}")))?;

        self.jobs.lock().insert(
            macro_id.clone(),
            ScheduledJob {
                macro_id,
                cron: cron_expr.to_string(),
                job_uuid,
            },
        );

        Ok(job_uuid)
    }

    /// Remove a scheduled macro by its macro_id.
    pub async fn unschedule(&self, macro_id: &str) -> Result<(), CntrlError> {
        let job_uuid = {
            let mut jobs = self.jobs.lock();
            match jobs.remove(macro_id) {
                Some(j) => j.job_uuid,
                None => return Ok(()), // idempotent
            }
        };

        self.sched
            .remove(&job_uuid)
            .await
            .map_err(|e| CntrlError::Macro(format!("Failed to remove job: {e}")))?;

        Ok(())
    }

    /// List all currently scheduled macros.
    pub fn list_scheduled(&self) -> Vec<ScheduledJob> {
        self.jobs.lock().values().cloned().collect()
    }

    /// Returns `true` if the given macro_id is currently scheduled.
    pub fn is_scheduled(&self, macro_id: &str) -> bool {
        self.jobs.lock().contains_key(macro_id)
    }
}
