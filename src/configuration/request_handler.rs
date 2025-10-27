use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestHandler {
    pub id: String,                                  // Generated id, unique, so it can be referenced from sites as a handler
    pub is_enabled: bool,                            // Whether it is enabled or not
    pub name: String,                                // A name to identify the handler, self chosen
    pub handler_type: String,                        // e.g., "php", "python", etc. Used by the handlers to identify if they should handle requests
    pub request_timeout: usize,                      // Seconds
    pub concurrent_threads: usize,                   // 0 = automatically based on CPU cores on this machine - If PHP-FPM or similar is used, this should match the max children configured there
    pub file_match: Vec<String>,                     // .php, .html, etc
    pub executable: String,                          // Path to the executable or script that handles the request, like php-cgi.exe location for PHP on windows
    pub ip_and_port: String,                         // IP and port to connect to the handler, e.g. 127.0.0.1:9000 for FastCGI passthrough
    pub other_webroot: String,                       // Optional webroot to use when passing to the handler, if different from the site's webroot
    pub extra_handler_config: Vec<(String, String)>, // Key/value pairs for extra handler configuration
    pub extra_environment: Vec<(String, String)>,    // Key/value pairs to add to environment, passed on to the handler
}

impl RequestHandler {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate ID
        if self.id.trim().is_empty() {
            errors.push("Request handler ID cannot be empty".to_string());
        } else if !self.id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            errors.push("Request handler ID can only contain alphanumeric characters, underscores, and hyphens".to_string());
        }

        // Validate name
        if self.name.trim().is_empty() {
            errors.push("Request handler name cannot be empty".to_string());
        }

        // Validate handler type
        if self.handler_type.trim().is_empty() {
            errors.push("Handler type cannot be empty".to_string());
        } else {
            // Validate known handler types
            let valid_types = ["php", "python", "node", "static", "proxy"];
            if !valid_types.contains(&self.handler_type.trim()) {
                errors.push(format!("Unknown handler type '{}'. Valid types are: {}", self.handler_type, valid_types.join(", ")));
            }
        }

        // Validate request timeout
        if self.request_timeout == 0 {
            errors.push("Request timeout cannot be 0 seconds".to_string());
        } else if self.request_timeout > 3600 {
            errors.push("Request timeout cannot exceed 3600 seconds (1 hour)".to_string());
        }

        // Validate max concurrent threads
        if self.concurrent_threads > 1000 {
            errors.push("Max concurrent threads cannot exceed 1000".to_string());
        }

        // Validate file match patterns
        if self.file_match.is_empty() {
            errors.push("File match patterns cannot be empty".to_string());
        } else {
            for (pattern_idx, pattern) in self.file_match.iter().enumerate() {
                if pattern.trim().is_empty() {
                    errors.push(format!("File match pattern {} cannot be empty", pattern_idx + 1));
                } else if !pattern.starts_with('.') && !pattern.starts_with('*') {
                    errors.push(format!("File match pattern '{}' should start with '.' or '*'", pattern));
                }
            }
        }

        // Validate executable path
        if self.executable.trim().is_empty() {
            errors.push("Executable path cannot be empty".to_string());
        }

        // Validate IP and port
        if !self.ip_and_port.trim().is_empty() {
            // Basic format validation for IP:port
            if !self.ip_and_port.contains(':') {
                errors.push("IP and port must be in format 'IP:PORT'".to_string());
            } else {
                let parts: Vec<&str> = self.ip_and_port.split(':').collect();
                if parts.len() != 2 {
                    errors.push("IP and port must be in format 'IP:PORT'".to_string());
                } else {
                    // Validate IP part, which can be an IP address or hostname
                    if parts[0].contains('.') {
                        if parts[0].parse::<std::net::IpAddr>().is_err() {
                            errors.push(format!("Invalid IP address '{}': {}", self.ip_and_port, parts[0]));
                        }
                    }

                    // Validate port part
                    if let Ok(port) = parts[1].parse::<u16>() {
                        if port == 0 {
                            errors.push("Port cannot be 0".to_string());
                        }
                    } else {
                        errors.push(format!("Invalid port in '{}': {}", self.ip_and_port, parts[1]));
                    }
                }
            }
        }

        // Validate extra handler config
        for (config_idx, (key, value)) in self.extra_handler_config.iter().enumerate() {
            if key.trim().is_empty() {
                errors.push(format!("Extra handler config key {} cannot be empty", config_idx + 1));
            }
            if value.trim().is_empty() {
                errors.push(format!("Extra handler config value {} cannot be empty", config_idx + 1));
            }
        }

        // Validate extra environment variables
        for (env_idx, (key, value)) in self.extra_environment.iter().enumerate() {
            if key.trim().is_empty() {
                errors.push(format!("Environment variable key {} cannot be empty", env_idx + 1));
            }
            if value.trim().is_empty() {
                errors.push(format!("Environment variable value {} cannot be empty", env_idx + 1));
            }

            // Check for valid environment variable name format
            if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                errors.push(format!("Environment variable key '{}' can only contain alphanumeric characters and underscores", key));
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
