use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CoverageMetrics {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub total_requirements: usize,
    pub covered_requirements: usize,
    pub coverage_percentage: f64,
    pub uncovered_requirements: Vec<String>,
    pub file_coverage: HashMap<String, FileCoverage>,
    pub calculated_at: DateTime<Utc>,
}

impl CoverageMetrics {
    pub fn new(scan_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            scan_id,
            total_requirements: 0,
            covered_requirements: 0,
            coverage_percentage: 0.0,
            uncovered_requirements: Vec::new(),
            file_coverage: HashMap::new(),
            calculated_at: Utc::now(),
        }
    }

    pub fn calculate_coverage(&mut self, requirements: &HashMap<String, Vec<String>>) {
        self.total_requirements = requirements.len();
        self.covered_requirements = requirements.values()
            .filter(|files| !files.is_empty())
            .count();
        
        self.coverage_percentage = if self.total_requirements > 0 {
            (self.covered_requirements as f64 / self.total_requirements as f64) * 100.0
        } else {
            0.0
        };

        self.uncovered_requirements = requirements
            .iter()
            .filter(|(_, files)| files.is_empty())
            .map(|(req_id, _)| req_id.clone())
            .collect();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FileCoverage {
    pub file_path: String,
    pub requirement_count: usize,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CoverageReport {
    pub scan_id: Uuid,
    pub metrics: CoverageMetrics,
    pub recommendations: Vec<String>,
}

impl CoverageReport {
    pub fn generate_recommendations(metrics: &CoverageMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if metrics.coverage_percentage < 50.0 {
            recommendations.push(
                "Critically low requirement coverage. Immediate increase in annotations recommended.".to_string()
            );
        } else if metrics.coverage_percentage < 75.0 {
            recommendations.push(
                "Requirement coverage below target level. Consider adding annotations for key components.".to_string()
            );
        }

        if !metrics.uncovered_requirements.is_empty() {
            recommendations.push(format!(
                "Found {} uncovered requirements. Attention needed for these specifications.",
                metrics.uncovered_requirements.len()
            ));
        }

        recommendations
    }
}
