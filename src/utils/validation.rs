use crate::models::{ScanRequest, Result, SddNavigatorError};
use std::path::Path;

#[allow(dead_code)]
pub trait Validator<T> {
    fn validate(&self) -> Result<()>;
}

impl Validator<ScanRequest> for ScanRequest {
    fn validate(&self) -> Result<()> {
        if !self.path.exists() {
            return Err(SddNavigatorError::PathNotFound(
                self.path.to_string_lossy().to_string()
            ));
        }

        if self.file_patterns.is_empty() {
            return Err(SddNavigatorError::Config(
                "At least one file pattern must be specified".to_string()
            ));
        }

        for pattern in &self.file_patterns {
            if pattern.is_empty() {
                return Err(SddNavigatorError::Config(
                    "File patterns cannot be empty".to_string()
                ));
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub fn validate_requirement_id(requirement_id: &str) -> Result<()> {
    if requirement_id.is_empty() {
        return Err(SddNavigatorError::InvalidAnnotation(
            "Requirement ID cannot be empty".to_string()
        ));
    }

    if !requirement_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(SddNavigatorError::InvalidAnnotation(
            "Requirement ID can only contain alphanumeric characters, hyphens, and underscores".to_string()
        ));
    }

    if requirement_id.len() > 100 {
        return Err(SddNavigatorError::InvalidAnnotation(
            "Requirement ID too long (max 100 characters)".to_string()
        ));
    }

    Ok(())
}

#[allow(dead_code)]
pub fn validate_file_path(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(SddNavigatorError::PathNotFound(
            path.to_string_lossy().to_string()
        ));
    }

    if !path.is_file() {
        return Err(SddNavigatorError::Scan(
            "Path must be a file".to_string()
        ));
    }

    let metadata = std::fs::metadata(path)?;
    let file_size_mb = metadata.len() / (1024 * 1024);
    
    if file_size_mb > 50 {
        return Err(SddNavigatorError::Scan(
            format!("File too large: {}MB (max 50MB)", file_size_mb)
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_requirement_id_valid() {
        assert!(validate_requirement_id("REQ-001").is_ok());
        assert!(validate_requirement_id("REQ_001").is_ok());
        assert!(validate_requirement_id("REQ001").is_ok());
    }

    #[test]
    fn test_validate_requirement_id_invalid() {
        assert!(validate_requirement_id("").is_err());
        assert!(validate_requirement_id("REQ@001").is_err());
        assert!(validate_requirement_id("REQ-001!").is_err());
    }

    #[test]
    fn test_scan_request_validation() {
        let mut request = ScanRequest::default();
        
        request.path = PathBuf::from("/non/existent/path");
        assert!(request.validate().is_err());
        
        request.path = PathBuf::from(".");
        request.file_patterns.clear();
        assert!(request.validate().is_err());
    }
}
