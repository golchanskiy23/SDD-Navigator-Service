// @req(REQ-012) Data models for the application

use serde::{Deserialize, Serialize};

// @req(REQ-013) User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub is_active: bool,
}

impl User {
    // @req(REQ-014) User creation
    pub fn new(username: String, email: String) -> Self {
        Self {
            id: 0, // Will be set by database
            username,
            email,
            is_active: true,
        }
    }
    
    // @req(REQ-015) User validation
    pub fn validate(&self) -> Result<(), String> {
        if self.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        
        if !self.email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        
        Ok(())
    }
}

// @req(REQ-016) Session model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl Session {
    // @req(REQ-017) Session creation
    pub fn new(user_id: u32) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            created_at: now,
            expires_at: now + chrono::Duration::hours(24),
        }
    }
    
    // @req(REQ-018) Session validation
    pub fn is_valid(&self) -> bool {
        chrono::Utc::now() < self.expires_at
    }
}
