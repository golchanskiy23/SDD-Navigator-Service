use crate::models::{Result, RequirementAnnotation, AnnotationPattern};
use std::path::Path;
use regex::Regex;
use std::fs;

pub trait FileProcessor: Send + Sync {
    fn process_file(&self, file_path: &Path) -> Result<Vec<RequirementAnnotation>>;
    fn supports_file(&self, file_path: &Path) -> bool;
}

pub struct DefaultFileProcessor {
    #[allow(dead_code)]
    pattern: AnnotationPattern,
    regex: Regex,
}

impl DefaultFileProcessor {
    pub fn new(pattern: AnnotationPattern) -> Result<Self> {
        let regex = Regex::new(&pattern.pattern)
            .map_err(|e| crate::models::SddNavigatorError::Regex(e))?;
        
        Ok(Self { pattern, regex })
    }
}

impl Default for DefaultFileProcessor {
    fn default() -> Self {
        Self::new(AnnotationPattern::default())
            .expect("Default pattern should be valid")
    }
}

impl FileProcessor for DefaultFileProcessor {
    fn process_file(&self, file_path: &Path) -> Result<Vec<RequirementAnnotation>> {
        let content = fs::read_to_string(file_path)?;
        let mut annotations = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            for captures in self.regex.captures_iter(line) {
                if let Some(req_match) = captures.get(1) {
                    let annotation = RequirementAnnotation::new(
                        req_match.as_str().to_string(),
                        file_path.to_path_buf(),
                        line_num + 1,
                        line.to_string(),
                        Some(extract_context(&content, line_num)),
                    );
                    annotations.push(annotation);
                }
            }
        }
        
        Ok(annotations)
    }

    fn supports_file(&self, file_path: &Path) -> bool {
        if let Some(extension) = file_path.extension() {
            matches!(
                extension.to_str(),
                Some("rs") | Some("js") | Some("ts") | Some("java") |
                Some("py") | Some("cpp") | Some("c") | Some("h") |
                Some("hpp") | Some("cc") | Some("cxx")
            )
        } else {
            false
        }
    }
}

fn extract_context(content: &str, line_num: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start = line_num.saturating_sub(2);
    let end = std::cmp::min(line_num + 3, lines.len());
    
    lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let actual_line_num = start + i + 1;
            if actual_line_num == line_num + 1 {
                format!(">>> {} | {}", actual_line_num, line)
            } else {
                format!("    {} | {}", actual_line_num, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
