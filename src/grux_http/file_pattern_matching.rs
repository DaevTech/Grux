use log::trace;
use wildcard::{Wildcard, WildcardBuilder};

use crate::grux_configuration::get_configuration;
use std::sync::OnceLock;

pub struct BlockedFilePatternMatching<'a> {
    pub wildcards: Vec<Wildcard<'a>>,
}

impl<'a> BlockedFilePatternMatching<'a> {
    pub fn new() -> Self {
        let config = get_configuration();
        trace!("Initializing blocked file pattern matching with patterns: {:?}", config.core.server_settings.blocked_file_patterns);
        let wildcards = config.core.server_settings.blocked_file_patterns
            .iter()
            .map(|p| {
                WildcardBuilder::new(p.as_bytes()).case_insensitive(true).build().unwrap()
            })
            .collect();

        BlockedFilePatternMatching { wildcards }
    }

    pub fn is_file_pattern_blocked(&self, file_name: &str) -> bool {
        trace!("Checking if file pattern is blocked for file: {}", file_name);
        for wc in &self.wildcards {
            if wc.is_match(file_name.as_bytes()) {
                trace!("File pattern matched blocked pattern: {:?}", wc.pattern());
                return true;
            }
        }
        false
    }
}

static BLOCKED_FILE_PATTERN_MATCHING_SINGLETON: OnceLock<BlockedFilePatternMatching> = OnceLock::new();

pub fn get_blocked_file_pattern_matching() -> &'static BlockedFilePatternMatching<'static> {
    BLOCKED_FILE_PATTERN_MATCHING_SINGLETON.get_or_init(|| BlockedFilePatternMatching::new())
}

pub struct WhitelistedFilePatternMatching<'a> {
    pub wildcards: Vec<Wildcard<'a>>,
}

impl<'a> WhitelistedFilePatternMatching<'a> {
    pub fn new() -> Self {
        let config = get_configuration();
        trace!("Initializing whitelisted file pattern matching with patterns: {:?}", config.core.server_settings.whitelisted_file_patterns);
        let wildcards = config.core.server_settings.whitelisted_file_patterns
            .iter()
            .map(|p| {
                WildcardBuilder::new(p.as_bytes()).case_insensitive(true).build().unwrap()
            })
            .collect();

        WhitelistedFilePatternMatching { wildcards }
    }

    pub fn is_file_pattern_whitelisted(&self, file_name: &str) -> bool {
        trace!("Checking if file pattern is whitelisted for file: {}", file_name);
        for wc in &self.wildcards {
            if wc.is_match(file_name.as_bytes()) {
                trace!("File pattern matched whitelisted pattern: {:?}", wc.pattern());
                return true;
            }
        }
        false
    }
}

static WHITELISTED_FILE_PATTERN_MATCHING_SINGLETON: OnceLock<WhitelistedFilePatternMatching> = OnceLock::new();

pub fn get_whitelisted_file_pattern_matching() -> &'static WhitelistedFilePatternMatching<'static> {
    WHITELISTED_FILE_PATTERN_MATCHING_SINGLETON.get_or_init(|| WhitelistedFilePatternMatching::new())
}