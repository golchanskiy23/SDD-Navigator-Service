// @req(REQ-004) Utility functions for data manipulation

pub mod validators {
    // @req(REQ-006) Input validation
    pub fn validate_email(email: &str) -> bool {
        email.contains('@') && email.contains('.')
    }
    
    // @req(REQ-007) Password strength validation
    pub fn validate_password(password: &str) -> bool {
        password.len() >= 8 && 
        password.chars().any(|c| c.is_numeric()) &&
        password.chars().any(|c| c.is_uppercase())
    }
}

pub mod formatters {
    // @req(REQ-008) String formatting utilities
    pub fn format_username(username: &str) -> String {
        username.trim().to_lowercase()
    }
    
    // @req(REQ-009) Date formatting
    pub fn format_date(date: chrono::NaiveDate) -> String {
        date.format("%Y-%m-%d").to_string()
    }
}
