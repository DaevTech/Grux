use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerSettings {
    pub max_body_size: usize, // in bytes
    pub blocked_file_patterns: Vec<String>,
    pub whitelisted_file_patterns: Vec<String>,
}

impl ServerSettings {
    pub fn sanitize(&mut self) {


    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate max_body_size
        if self.max_body_size == 0 {
            errors.push("Max body size cannot be 0".to_string());
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}