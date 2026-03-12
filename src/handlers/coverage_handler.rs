use crate::models::{CoverageReport, CoverageMetrics};
use crate::services::{CoverageCalculator, DefaultCoverageCalculator};
use crate::repository::{ScanRepository, InMemoryScanRepository};
use axum::{
    extract::{Path, State},
    response::Json,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct CoverageHandler {
    calculator: DefaultCoverageCalculator,
    repository: InMemoryScanRepository,
}

impl CoverageHandler {
    pub fn new(calculator: DefaultCoverageCalculator, repository: InMemoryScanRepository) -> Self {
        Self { calculator, repository }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/scans/{scan_id}/coverage",
    responses(
        (status = 200, description = "Coverage metrics for scan", body = CoverageMetrics),
        (status = 404, description = "Scan not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("scan_id" = Uuid, Path, description = "Scan ID")
    ),
    tag = "coverage"
)]
#[axum::debug_handler]
pub async fn get_coverage_metrics(
    State(app_state): State<crate::AppState>,
    Path(scan_id): Path<Uuid>,
) -> std::result::Result<Json<CoverageMetrics>, crate::handlers::ApiError> {
    let scan_result: std::result::Result<Option<crate::models::ScanResult>, crate::models::SddNavigatorError> = app_state.coverage_handler.repository.get_scan(scan_id)
        .await;
    
    let scan = scan_result.map_err(|e| crate::handlers::ApiError::DatabaseError(e.to_string()))?
        .ok_or_else(|| crate::handlers::ApiError::NotFound(format!("Scan {} not found", scan_id)))?;
    
    let metrics: crate::models::CoverageMetrics = app_state.coverage_handler.calculator.calculate(scan_id, &scan.annotations)
        .map_err(|e| crate::handlers::ApiError::CoverageError(e.to_string()))?;
    
    Ok(Json(metrics))
}

#[utoipa::path(
    get,
    path = "/api/v1/scans/{scan_id}/coverage/report",
    responses(
        (status = 200, description = "Coverage report for scan", body = CoverageReport),
        (status = 404, description = "Scan not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("scan_id" = Uuid, Path, description = "Scan ID")
    ),
    tag = "coverage"
)]
#[axum::debug_handler]
pub async fn get_coverage_report(
    State(app_state): State<crate::AppState>,
    Path(scan_id): Path<Uuid>,
) -> std::result::Result<Json<CoverageReport>, crate::handlers::ApiError> {
    let scan_result: std::result::Result<Option<crate::models::ScanResult>, crate::models::SddNavigatorError> = app_state.coverage_handler.repository.get_scan(scan_id)
        .await;
    
    let scan = scan_result.map_err(|e| crate::handlers::ApiError::DatabaseError(e.to_string()))?
        .ok_or_else(|| crate::handlers::ApiError::NotFound(format!("Scan {} not found", scan_id)))?;
    
    let metrics: crate::models::CoverageMetrics = app_state.coverage_handler.calculator.calculate(scan_id, &scan.annotations)
        .map_err(|e| crate::handlers::ApiError::CoverageError(e.to_string()))?;
    
    let report: crate::models::CoverageReport = app_state.coverage_handler.calculator.generate_report(metrics)
        .map_err(|e| crate::handlers::ApiError::CoverageError(e.to_string()))?;
    
    Ok(Json(report))
}
