use crate::models::{Result, ScanResult, ScanRequest, ScanStatus};
use crate::services::{FileProcessor, DefaultFileProcessor};
use std::path::Path;
use walkdir::{WalkDir, DirEntry};
use std::sync::Arc;

pub trait Scanner: Send + Sync {
    fn scan(&self, request: ScanRequest) -> Result<ScanResult>;
}

#[derive(Clone)]
pub struct FileSystemScanner {
    file_processor: Arc<dyn FileProcessor>,
}

impl FileSystemScanner {
    pub fn new(file_processor: Arc<dyn FileProcessor>) -> Self {
        Self { file_processor }
    }

    pub fn with_default_processor() -> Self {
        Self::new(Arc::new(DefaultFileProcessor::default()))
    }

    fn should_include_file(&self, entry: &DirEntry, request: &ScanRequest) -> bool {
        if !entry.file_type().is_file() {
            return false;
        }

        let path = entry.path();
        
        if !self.file_processor.supports_file(path) {
            return false;
        }

        if !request.include_tests && is_test_file(path) {
            return false;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            return request.file_patterns.iter().any(|pattern| {
                match glob::Pattern::new(pattern) {
                    Ok(pat) => pat.matches(file_name),
                    Err(_) => file_name.contains(pattern),
                }
            });
        }

        false
    }
}

impl Scanner for FileSystemScanner {
    fn scan(&self, request: ScanRequest) -> Result<ScanResult> {
        let mut scan_result = ScanResult::new(request.path.clone());
        
        if !request.path.exists() {
            scan_result.fail(format!("Path does not exist: {:?}", request.path));
            return Ok(scan_result);
        }

        let walk_dir = if request.recursive {
            WalkDir::new(&request.path)
        } else {
            WalkDir::new(&request.path).max_depth(1)
        };

        let mut annotations = Vec::new();
        let mut processed_files = 0;
        let mut failed_files = 0;

        for entry in walk_dir.into_iter().filter_map(|e| e.ok()) {
            if self.should_include_file(&entry, &request) {
                match self.file_processor.process_file(entry.path()) {
                    Ok(file_annotations) => {
                        processed_files += 1;
                        annotations.extend(file_annotations);
                    }
                    Err(e) => {
                        failed_files += 1;
                        eprintln!("Failed to process file {:?}: {}", entry.path(), e);
                    }
                }
            }
        }

        scan_result.annotations = annotations;
        
        let summary = format!(
            "Processed {} files, found {} annotations, {} files failed",
            processed_files,
            scan_result.annotations.len(),
            failed_files
        );
        
        scan_result.status = ScanStatus::Completed;
        scan_result.completed_at = Some(chrono::Utc::now());
        
        println!("Scan completed: {}", summary);
        
        Ok(scan_result)
    }
}

fn is_test_file(path: &Path) -> bool {
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        return file_name.contains("test") || 
               file_name.starts_with("test_") ||
               file_name.ends_with("_test") ||
               file_name.ends_with(".test");
    }
    false
}
