use serde::{Deserialize, Serialize};
use crate::configuration::site::Site;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Binding {
    pub id: usize,
    pub ip: String,
    pub port: u16,
    pub is_admin: bool,
    pub is_tls: bool,
    #[serde(skip)]
    pub sites: Vec<Site>,
}

impl Binding {
    pub fn get_sites(&self) -> &Vec<Site> {
        &self.sites
    }

    pub fn add_site(&mut self, site: Site) {
        self.sites.push(site);
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate IP address
        if self.ip.is_empty() {
            errors.push("IP address cannot be empty".to_string());
        } else if self.ip.parse::<std::net::IpAddr>().is_err() {
            errors.push(format!("Invalid IP address: {}", self.ip));
        }

        // Validate port
        if self.port == 0 {
            errors.push("Port cannot be 0".to_string());
        }

        // Validate common TLS port usage
        if self.is_tls && self.port == 80 {
            errors.push("Port 80 is typically used for HTTP, not HTTPS. Consider using port 443 for TLS".to_string());
        }
        if !self.is_tls && self.port == 443 {
            errors.push("Port 443 is typically used for HTTPS, not HTTP. Consider using port 80 for non-TLS or enable TLS".to_string());
        }

        // Admin binding specific validations
        if self.is_admin {
            // Admin bindings should typically use TLS for security
            if !self.is_tls {
                errors.push("Admin binding should use TLS for security".to_string());
            }

            // Admin bindings should have at least one site
            if self.sites.is_empty() {
                errors.push("Admin binding must have at least one site configured".to_string());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
