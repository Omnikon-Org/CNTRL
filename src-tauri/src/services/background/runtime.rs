use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::{mpsc, Semaphore};

use super::error::BackgroundError;
use super::task::BackgroundTask;
use super::worker::execute_task;
use crate::services::browser::BrowserService;

pub struct BackgroundRuntime {
    sender: mpsc::Sender<BackgroundTask>,
}

impl BackgroundRuntime {
    pub fn new(app: AppHandle, browser: BrowserService, max_workers: usize, queue_capacity: usize) -> Self {
        let (sender, mut receiver) = mpsc::channel::<BackgroundTask>(queue_capacity);
        let semaphore = Arc::new(Semaphore::new(max_workers));

        tauri::async_runtime::spawn(async move {
            while let Some(task) = receiver.recv().await {
                let sem = semaphore.clone();
                let app_clone = app.clone();
                let browser_clone = browser.clone();

                tauri::async_runtime::spawn(async move {
                    // Acquire worker slot
                    if let Ok(_permit) = sem.acquire().await {
                        // Execute task (timeout and cleanup is handled inside execute_task)
                        let _ = execute_task(&app_clone, browser_clone, task).await;
                        // Permit is released when _permit goes out of scope here
                    }
                });
            }
        });

        Self { sender }
    }

    pub async fn enqueue(&self, task: BackgroundTask) -> Result<(), BackgroundError> {
        self.sender
            .send(task)
            .await
            .map_err(|_| BackgroundError::InternalError("Runtime queue closed".into()))
    }
}
