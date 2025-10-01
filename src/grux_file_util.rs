use std::env;
use std::path::{Component, Path, PathBuf};

/// Sanitizes and resolves a file path into an absolute path.
/// - Expands relative paths to absolute.
/// - Normalizes separators to `/`.
/// - Cleans up `.` and `..`.
/// - Removes duplicate separators.
/// Works on both Windows and Unix.
pub fn get_full_file_path<P: AsRef<Path>>(input: P) -> std::io::Result<String> {
    let mut path = PathBuf::new();

    let input_path = input.as_ref();

    // If relative, start from current dir
    if input_path.is_relative() {
        path.push(env::current_dir()?);
    }

    path.push(input_path);

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
/// - If `path_str` starts with `base_path`, returns the relative directory (with forward slashes, no leading slash) and file name.
/// - If not, returns ("", file_name).
pub fn split_path(base_path: &str, path_str: &str) -> (String, String) {
    let base = Path::new(base_path).components().collect::<PathBuf>();
    let path = Path::new(path_str);

    // If path_str starts with base_path, strip base_path prefix
    let rel = match path.strip_prefix(&base) {
        Ok(rel) => rel,
        Err(_) => path,
    };

    let file = rel.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .replace('\\', "/");

    let dir = rel.parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|| "".to_string());

    (dir.trim_start_matches('/').to_string(), file)
}

// We expect all web roots to be cleaned, with forward slashes and absolute paths and should be able to handle replacing webroot from Windows to Unix style paths and vice versa
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