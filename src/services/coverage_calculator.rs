use crate::models::{Result, CoverageMetrics, CoverageReport, RequirementAnnotation};
use std::collections::HashMap;
use uuid::Uuid;

pub trait CoverageCalculator: Send + Sync {
    fn calculate(&self, scan_id: Uuid, annotations: &[RequirementAnnotation]) -> Result<CoverageMetrics>;
    fn generate_report(&self, metrics: CoverageMetrics) -> Result<CoverageReport>;
}

#[derive(Clone)]
pub struct DefaultCoverageCalculator;

impl DefaultCoverageCalculator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultCoverageCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl CoverageCalculator for DefaultCoverageCalculator {
    fn calculate(&self, scan_id: Uuid, annotations: &[RequirementAnnotation]) -> Result<CoverageMetrics> {
        let mut metrics = CoverageMetrics::new(scan_id);
        
        let mut requirement_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut file_requirement_map: HashMap<String, Vec<String>> = HashMap::new();
        
        for annotation in annotations {
            requirement_map
                .entry(annotation.requirement_id.clone())
                .or_default()
                .push(annotation.file_path.to_string_lossy().to_string());
            
            let file_path = annotation.file_path.to_string_lossy().to_string();
            file_requirement_map
                .entry(file_path.clone())
                .or_default()
                .push(annotation.requirement_id.clone());
        }
        
        metrics.calculate_coverage(&requirement_map);
        
        for (file_path, requirements) in file_requirement_map {
            let unique_requirements: std::collections::HashSet<_> = requirements.iter().collect();
            metrics.file_coverage.insert(
                file_path.clone(),
                crate::models::FileCoverage {
                    file_path,
                    requirement_count: unique_requirements.len(),
                    requirements: unique_requirements.into_iter().cloned().collect(),
                },
            );
        }
        
        Ok(metrics)
    }

    fn generate_report(&self, metrics: CoverageMetrics) -> Result<CoverageReport> {
        let recommendations = CoverageReport::generate_recommendations(&metrics);
        
        Ok(CoverageReport {
            scan_id: metrics.scan_id,
            metrics,
            recommendations,
        })
    }
}
