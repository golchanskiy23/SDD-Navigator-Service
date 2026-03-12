use crate::repository::{ScanRepository, InMemoryScanRepository};
use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub database_status: String,
}

#[derive(Clone)]
pub struct HealthHandler {
    repository: InMemoryScanRepository,
}

impl HealthHandler {
    pub fn new(repository: InMemoryScanRepository) -> Self {
        Self { repository }
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check", body = HealthResponse)
    ),
    tag = "health"
)]
#[axum::debug_handler]
pub async fn health_check(
    State(app_state): State<crate::AppState>,
) -> std::result::Result<Json<HealthResponse>, crate::handlers::ApiError> {
    let db_status = match app_state.health_handler.repository.get_scan_statistics().await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };
    
    let overall_status = if db_status == "healthy" {
        "healthy"
    } else {
        "degraded"
    };
    
    Ok(Json(HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        database_status: db_status,
    }))
}
