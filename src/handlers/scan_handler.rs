use crate::models::{ScanRequest, ScanResult};
use crate::services::{Scanner, FileSystemScanner};
use crate::repository::{ScanRepository, InMemoryScanRepository};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct ScanHandler {
    scanner: FileSystemScanner,
    repository: InMemoryScanRepository,
}

impl ScanHandler {
    pub fn new(scanner: FileSystemScanner, repository: InMemoryScanRepository) -> Self {
        Self { scanner, repository }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/scans",
    request_body = ScanRequest,
    responses(
        (status = 200, description = "Scan started successfully", body = ScanResult),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "scans"
)]
#[axum::debug_handler]
pub async fn create_scan(
    State(app_state): State<crate::AppState>,
    Json(request): Json<ScanRequest>,
) -> std::result::Result<Json<ScanResult>, crate::handlers::ApiError> {
    let scan_result: crate::models::ScanResult = app_state.scan_handler.scanner.scan(request)
        .map_err(|e| crate::handlers::ApiError::ScanError(e.to_string()))?;
    
    let _scan_id = scan_result.id;
    app_state.scan_handler.repository.save_scan(&scan_result).await
        .map_err(|e| crate::handlers::ApiError::DatabaseError(e.to_string()))?;
    
    Ok(Json(scan_result))
}

#[utoipa::path(
    get,
    path = "/api/v1/scans/{scan_id}",
    responses(
        (status = 200, description = "Scan details", body = ScanResult),
        (status = 404, description = "Scan not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("scan_id" = Uuid, Path, description = "Scan ID")
    ),
    tag = "scans"
)]
#[axum::debug_handler]
pub async fn get_scan(
    State(app_state): State<crate::AppState>,
    Path(scan_id): Path<Uuid>,
) -> std::result::Result<Json<ScanResult>, crate::handlers::ApiError> {
    let scan_result: std::result::Result<Option<crate::models::ScanResult>, crate::models::SddNavigatorError> = app_state.scan_handler.repository.get_scan(scan_id)
        .await;
    
    let scan = scan_result.map_err(|e| crate::handlers::ApiError::DatabaseError(e.to_string()))?
        .ok_or_else(|| crate::handlers::ApiError::NotFound(format!("Scan {} not found", scan_id)))?;
    
    Ok(Json(scan))
}

#[utoipa::path(
    get,
    path = "/api/v1/scans",
    responses(
        (status = 200, description = "List of all scans", body = Vec<ScanResult>),
        (status = 500, description = "Internal server error")
    ),
    tag = "scans"
)]
#[axum::debug_handler]
pub async fn list_scans(
    State(app_state): State<crate::AppState>,
) -> std::result::Result<Json<Vec<ScanResult>>, crate::handlers::ApiError> {
    let scans: std::result::Result<Vec<crate::models::ScanResult>, crate::models::SddNavigatorError> = app_state.scan_handler.repository.get_all_scans()
        .await;
    
    let scans = scans.map_err(|e| crate::handlers::ApiError::DatabaseError(e.to_string()))?;
    
    Ok(Json(scans))
}

#[utoipa::path(
    delete,
    path = "/api/v1/scans/{scan_id}",
    responses(
        (status = 204, description = "Scan deleted successfully"),
        (status = 404, description = "Scan not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("scan_id" = Uuid, Path, description = "Scan ID")
    ),
    tag = "scans"
)]
#[axum::debug_handler]
pub async fn delete_scan(
    State(app_state): State<crate::AppState>,
    Path(scan_id): Path<Uuid>,
) -> std::result::Result<StatusCode, crate::handlers::ApiError> {
    let deleted: std::result::Result<bool, crate::models::SddNavigatorError> = app_state.scan_handler.repository.delete_scan(scan_id)
        .await;
    
    let deleted = deleted.map_err(|e| crate::handlers::ApiError::DatabaseError(e.to_string()))?;
    
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(crate::handlers::ApiError::NotFound(format!("Scan {} not found", scan_id)))
    }
}
