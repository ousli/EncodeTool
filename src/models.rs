use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Log {
        level: String,
        message: String,
    },
    Progress {
        file: String,
        file_index: usize,
        file_total: usize,
        file_percent: f64,
        global_percent: f64,
        eta: String,
    },
    FileDone {
        file: String,
        output: String,
    },
    Done {
        export: String,
    },
    Error {
        message: String,
        code: String,
    },
}

#[derive(Debug, Clone)]
pub struct ProcessConfig {
    pub dry_run: bool,
    pub jsonl: bool,
    pub overwrite: bool,
    pub source: PathBuf,
    pub export: PathBuf,
}
