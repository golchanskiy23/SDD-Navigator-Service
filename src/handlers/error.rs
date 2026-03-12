use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Scan error: {0}")]
    ScanError(String),
    
    #[error("Coverage error: {0}")]
    CoverageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::ScanError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::CoverageError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::SerializationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::SerializationError(err.to_string())
    }
}

impl From<crate::models::SddNavigatorError> for ApiError {
    fn from(err: crate::models::SddNavigatorError) -> Self {
        match err {
            crate::models::SddNavigatorError::Database(db_err) => ApiError::DatabaseError(db_err.to_string()),
            crate::models::SddNavigatorError::Serialization(ser_err) => ApiError::SerializationError(ser_err.to_string()),
            crate::models::SddNavigatorError::Scan(scan_err) => ApiError::ScanError(scan_err),
            crate::models::SddNavigatorError::CoverageCalculation(coverage_err) => ApiError::CoverageError(coverage_err),
            crate::models::SddNavigatorError::PathNotFound(path) => ApiError::NotFound(format!("Path not found: {}", path)),
            crate::models::SddNavigatorError::InvalidAnnotation(msg) => ApiError::BadRequest(format!("Invalid annotation: {}", msg)),
            _ => ApiError::Internal(err.to_string()),
        }
    }
}
