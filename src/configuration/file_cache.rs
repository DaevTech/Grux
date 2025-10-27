use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileCache {
    pub is_enabled: bool,
    pub cache_item_size: usize,
    pub cache_max_size_per_file: usize,
    pub cache_item_time_between_checks: usize,
    pub cleanup_thread_interval: usize,
    pub max_item_lifetime: usize,         // in seconds
    pub forced_eviction_threshold: usize, // 1-99 %
}

impl FileCache {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate cache_item_size
        if self.cache_item_size == 0 {
            errors.push("Max cached items count cannot be 0".to_string());
        }

        // Validate cache_max_size_per_file
        if self.cache_max_size_per_file == 0 {
            errors.push("Max size per file cannot be 0 bytes".to_string());
        }

        // Validate cache_item_time_between_checks
        if self.cache_item_time_between_checks == 0 {
            errors.push("Cache item time between checks cannot be 0".to_string());
        }

        // Validate cleanup_thread_interval
        if self.cleanup_thread_interval == 0 {
            errors.push("Cleanup thread interval cannot be 0".to_string());
        }

        // Validate max_item_lifetime
        if self.max_item_lifetime == 0 {
            errors.push("Max item lifetime cannot be 0".to_string());
        }

        // Validate forced_eviction_threshold (should be between 1-99)
        if self.forced_eviction_threshold == 0 || self.forced_eviction_threshold > 99 {
            errors.push("Forced eviction threshold must be between 1-99%".to_string());
        }

        // Note: cache_item_size is a count of items, cache_max_size_per_file is bytes per file
        // These are different units and cannot be compared directly

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
