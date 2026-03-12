// @req(REQ-010) Library entry point

pub mod utils;
pub mod models;

// @req(REQ-011) Core library functionality
pub fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing SDD Navigator library...");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // @req(TEST-001) Test library initialization
    #[test]
    fn test_initialization() {
        assert!(initialize().is_ok());
    }
    
    // @req(TEST-002) Test utility functions
    #[test]
    fn test_email_validation() {
        assert!(crate::utils::validators::validate_email("test@example.com"));
        assert!(!crate::utils::validators::validate_email("invalid-email"));
    }
}
