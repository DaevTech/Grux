use std::env;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;
use cached::proc_macro::cached;
use log::trace;

use crate::http::file_pattern_matching::{get_blocked_file_pattern_matching, get_whitelisted_file_pattern_matching};

/// Sanitizes and resolves a file path into an absolute path.
/// - Expands relative paths to absolute.
/// - Normalizes separators to `/`.
/// - Cleans up `.` and `..`.
/// - Removes duplicate separators.
/// Works on both Windows and Unix.
#[cached(
    size = 100,
    time = 10, // Cache for 10 seconds
    result = true,
    key = "String",
    convert = r#"{ input_path.to_string() }"#
)]
pub fn get_full_file_path(input_path: &String) -> Result<String, std::io::Error> {
    let mut path = PathBuf::new();

    // If relative, start from current dir
    if Path::new(&input_path).is_relative() {
        let current_dir_result = env::current_dir()?;
        path.push(current_dir_result);
    }

    path.push(&input_path);

    // Normalize components manually
    let mut normalized = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {
                // Skip "."
            }
            Component::ParentDir => {
                // Skip ".."
            }
            other => normalized.push(other),
        }
    }

    // Convert to string and normalize slashes
    let mut result = normalized
        .to_string_lossy()
        .replace('\\', "/");

    // Remove duplicate slashes (// → /)
    while result.contains("//") {
        result = result.replace("//", "/");
    }

    Ok(result)
}

/// Splits `path_str` into (relative_dir, file_name) based on `base_path`.
/// - If `path_str` starts with `base_path`, returns (base_path, remaining_path).
/// - If not, returns ("", path_str).
pub fn split_path(base_path: &str, path_str: &str) -> (String, String) {
    let base_path_cleaned = base_path.replace('\\', "/").trim_end_matches('/').to_string();
    let path_str_cleaned = path_str.replace('\\', "/");

    if path_str_cleaned.starts_with(&base_path_cleaned) {
        let remaining = &path_str_cleaned[base_path_cleaned.len()..];
        let remaining = remaining.trim_start_matches('/'); // Remove leading slash if present
        (base_path_cleaned, remaining.to_string())
    } else {
        ("".to_string(), path_str_cleaned)
    }
}

// We expect all web roots to be cleaned, with forward slashes and absolute paths and should be able to handle replacing webroot from Windows to Unix style paths and vice versa
#[cached(
    size = 100,
    time = 10, // Cache for 10 seconds
    key = "String",
    convert = r#"{ format!("{}|{}|{}", original_path, old_web_root, new_web_root) }"#
)]
pub fn replace_web_root_in_path(original_path: &str, old_web_root: &str, new_web_root: &str) -> String {
    let old_web_root_cleaned = old_web_root.replace('\\', "/").trim_end_matches('/').to_string();
    let new_web_root_cleaned = new_web_root.replace('\\', "/").trim_end_matches('/').to_string();

    if original_path.starts_with(&old_web_root_cleaned) {
        let relative_part = &original_path[old_web_root_cleaned.len()..];
        let relative_part = relative_part.trim_start_matches('/'); // Remove leading slash if present
        if relative_part.is_empty() {
            new_web_root_cleaned.clone()
        } else {
            format!("{}/{}", new_web_root_cleaned, relative_part)
        }
    } else {
        original_path.to_string() // Return original if it doesn't start with old web root
    }
}

// Check that the path is secure, by these tests:
// - The path starts with the base path, to prevent directory traversal attacks
// - The path does not contain any of the blocked file patterns
pub fn check_path_secure(base_path: &str, test_path: &str) -> bool {
    // Check that the test_path starts with the base_path
    let base_path_cleaned = base_path.replace('\\', "/").trim_end_matches('/').to_string();
    let test_path_cleaned = test_path.replace('\\', "/");
    if !test_path_cleaned.starts_with(&base_path_cleaned) {
        trace!("Path is blocked, as it does not start with the web root: {} file: {}", base_path_cleaned, test_path_cleaned);
        return false;
    }

    let (_path, file) = split_path(&base_path_cleaned, &test_path_cleaned);

    trace!("Check if file pattern is blocked or whitelisted: {}", &file);

    // Check if it is whitelisted first
    let pattern_whitelisting = get_whitelisted_file_pattern_matching();
    if pattern_whitelisting.is_file_pattern_whitelisted(&test_path_cleaned) {
        trace!("File pattern is whitelisted: {}", &test_path_cleaned);
        return true;
    }

    // Check the blacklisted file patterns
    let pattern_blocking = get_blocked_file_pattern_matching();
    if pattern_blocking.is_file_pattern_blocked(&file) {
        trace!("File pattern is blocked: {}", &file);
        return false;
    }

    true
}