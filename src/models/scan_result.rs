use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use utoipa::ToSchema;
use super::{RequirementAnnotation, CoverageMetrics};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScanResult {
    pub id: Uuid,
    pub scan_path: PathBuf,
    pub annotations: Vec<RequirementAnnotation>,
    pub metrics: Option<CoverageMetrics>,
    pub status: ScanStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl ScanResult {
    pub fn new(scan_path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            scan_path,
            annotations: Vec::new(),
            metrics: None,
            status: ScanStatus::InProgress,
            started_at: Utc::now(),
            completed_at: None,
            error_message: None,
        }
    }

    #[allow(dead_code)]
    pub fn complete(&mut self, metrics: CoverageMetrics) {
        self.metrics = Some(metrics);
        self.status = ScanStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    pub fn fail(&mut self, error: String) {
        self.status = ScanStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
    }

    #[allow(dead_code)]
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.completed_at.map(|end| end - self.started_at)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum ScanStatus {
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScanRequest {
    pub path: PathBuf,
    pub file_patterns: Vec<String>,
    pub include_tests: bool,
    pub recursive: bool,
}

impl Default for ScanRequest {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            file_patterns: vec![
                "*.rs".to_string(),
                "*.js".to_string(),
                "*.ts".to_string(),
                "*.java".to_string(),
                "*.py".to_string(),
                "*.cpp".to_string(),
                "*.c".to_string(),
                "*.h".to_string(),
            ],
            include_tests: true,
            recursive: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScanStatistics {
    pub total_scans: usize,
    pub successful_scans: usize,
    pub failed_scans: usize,
    pub average_coverage: f64,
    pub last_scan_time: Option<DateTime<Utc>>,
}
