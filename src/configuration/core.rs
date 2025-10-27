use serde::{Deserialize, Serialize};
use crate::configuration::file_cache::FileCache;
use crate::configuration::gzip::Gzip;
use crate::configuration::server_settings::ServerSettings;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Core {
    pub file_cache: FileCache,
    pub gzip: Gzip,
    pub server_settings: ServerSettings,
}

impl Core {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate file cache settings
        if let Err(file_cache_errors) = self.file_cache.validate() {
            for error in file_cache_errors {
                errors.push(format!("File Cache: {}", error));
            }
        }

        // Validate gzip settings
        if let Err(gzip_errors) = self.gzip.validate() {
            for error in gzip_errors {
                errors.push(format!("Gzip: {}", error));
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
