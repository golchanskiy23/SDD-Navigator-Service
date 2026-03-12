use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct RequirementAnnotation {
    pub id: Uuid,
    pub requirement_id: String,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub content: String,
    pub context: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl RequirementAnnotation {
    pub fn new(
        requirement_id: String,
        file_path: PathBuf,
        line_number: usize,
        content: String,
        context: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            requirement_id,
            file_path,
            line_number,
            content,
            context,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnnotationPattern {
    pub pattern: String,
    pub description: String,
}

impl Default for AnnotationPattern {
    fn default() -> Self {
        Self {
            pattern: r"@req\(([^)]+)\)".to_string(),
            description: "Matches @req(requirement_id) annotations".to_string(),
        }
    }
}
