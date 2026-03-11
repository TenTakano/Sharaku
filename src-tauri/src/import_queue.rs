use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::error::AppError;
use crate::importer::{self, ImportRequest};
use crate::settings;
use crate::AppDb;

pub struct ImportJob {
    pub id: String,
    pub library_id: String,
    pub library_root: Option<PathBuf>,
    pub requests: Vec<ImportRequest>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ImportQueueEvent {
    #[serde(rename_all = "camelCase")]
    Enqueued { job_id: String, total: usize },
    #[serde(rename_all = "camelCase")]
    JobStarted { job_id: String, total: usize },
    #[serde(rename_all = "camelCase")]
    Progress {
        job_id: String,
        current: usize,
        total: usize,
        title: String,
    },
    #[serde(rename_all = "camelCase")]
    ItemError {
        job_id: String,
        title: String,
        message: String,
    },
    #[serde(rename_all = "camelCase")]
    JobCompleted {
        job_id: String,
        succeeded: usize,
        failed: usize,
    },
    #[allow(dead_code)]
    #[serde(rename_all = "camelCase")]
    JobFailed { job_id: String, message: String },
}

pub struct ImportQueue {
    tx: mpsc::UnboundedSender<ImportJob>,
}

impl ImportQueue {
    pub fn new(app_handle: AppHandle, db: Arc<Mutex<AppDb>>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        tauri::async_runtime::spawn(worker(rx, app_handle, db));
        Self { tx }
    }

    pub fn enqueue(&self, job: ImportJob) -> Result<(), String> {
        self.tx
            .send(job)
            .map_err(|_| "取り込みキューの送信に失敗しました".to_string())
    }
}

async fn worker(
    mut rx: mpsc::UnboundedReceiver<ImportJob>,
    app_handle: AppHandle,
    db: Arc<Mutex<AppDb>>,
) {
    while let Some(job) = rx.recv().await {
        let total = job.requests.len();
        let _ = app_handle.emit(
            "import-queue",
            ImportQueueEvent::JobStarted {
                job_id: job.id.clone(),
                total,
            },
        );

        let mut succeeded = 0usize;
        let mut failed = 0usize;

        for (i, request) in job.requests.iter().enumerate() {
            let title = request.title.clone();
            let _ = app_handle.emit(
                "import-queue",
                ImportQueueEvent::Progress {
                    job_id: job.id.clone(),
                    current: i + 1,
                    total,
                    title: title.clone(),
                },
            );

            let db_clone = db.clone();
            let library_id = job.library_id.clone();
            let library_root = job.library_root.clone();
            let request = request.clone();

            let result = tokio::task::spawn_blocking(move || {
                let guard = db_clone.lock().unwrap();
                let lib_root = match library_root.as_deref() {
                    Some(path) => path,
                    None => {
                        let mode = settings::get_resource_mode(&guard.conn, &library_id)?;
                        if mode == "full" {
                            return Err(AppError::ImportError(
                                "ライブラリルートが設定されていません".to_string(),
                            ));
                        }
                        Path::new("")
                    }
                };
                importer::import_work(&request, &guard.conn, &library_id, lib_root)
            })
            .await;

            match result {
                Ok(Ok(_)) => succeeded += 1,
                Ok(Err(e)) => {
                    let _ = app_handle.emit(
                        "import-queue",
                        ImportQueueEvent::ItemError {
                            job_id: job.id.clone(),
                            title,
                            message: e.to_string(),
                        },
                    );
                    failed += 1;
                }
                Err(e) => {
                    let _ = app_handle.emit(
                        "import-queue",
                        ImportQueueEvent::ItemError {
                            job_id: job.id.clone(),
                            title,
                            message: e.to_string(),
                        },
                    );
                    failed += 1;
                }
            }
        }

        let _ = app_handle.emit(
            "import-queue",
            ImportQueueEvent::JobCompleted {
                job_id: job.id.clone(),
                succeeded,
                failed,
            },
        );
    }
}
