use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;

pub const ACME_CHALLENGE_PATH_PREFIX: &str = "/.well-known/acme-challenge/";

// Time after which a challenge entry is considered stale and can be cleaned up.
// ACME challenges typically complete within a few minutes, but we allow 1 hour for safety.
const CHALLENGE_EXPIRY_SECONDS: u64 = 3600;

/// A single HTTP-01 challenge entry containing the key authorization response.
#[derive(Clone, Debug)]
struct ChallengeEntry {
    /// The key authorization string to return for this challenge.
    /// Format: `{token}.{account_thumbprint}`
    key_authorization: String,
    /// When this challenge was added (for expiry tracking).
    created_at: Instant,
}

impl ChallengeEntry {
    fn new(key_authorization: String) -> Self {
        Self {
            key_authorization,
            created_at: Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > Duration::from_secs(CHALLENGE_EXPIRY_SECONDS)
    }
}

#[derive(Debug)]
pub struct AcmeHttp01ChallengeStore {
    challenges: DashMap<String, ChallengeEntry>,
}

impl AcmeHttp01ChallengeStore {
    pub fn new() -> Self {
        Self { challenges: DashMap::new() }
    }

    /// Add a challenge token with its key authorization.
    ///
    /// # Arguments
    /// * `token` - The challenge token from ACME
    /// * `key_authorization` - The key authorization string (`{token}.{thumbprint}`)
    pub fn add_challenge(&self, token: String, key_authorization: String) {
        self.challenges.insert(token, ChallengeEntry::new(key_authorization));
    }

    /// Remove a challenge token after it has been validated.
    ///
    /// # Arguments
    /// * `token` - The challenge token to remove
    pub fn remove_challenge(&self, token: &str) {
        self.challenges.remove(token);
    }

    /// Look up a challenge by token.
    /// Returns the key authorization if the token exists and hasn't expired.
    ///
    /// This is the hot path - optimized for fast lookups.
    ///
    /// # Arguments
    /// * `token` - The challenge token to look up
    #[inline]
    pub fn get_key_authorization(&self, token: &str) -> Option<String> {
        self.challenges
            .get(token)
            .and_then(|entry| if entry.is_expired() { None } else { Some(entry.key_authorization.clone()) })
    }

    /// Check if a request path is an ACME HTTP-01 challenge path and extract the token.
    ///
    /// This performs a fast prefix check before any allocation.
    ///
    /// # Arguments
    /// * `path` - The request path to check
    ///
    /// # Returns
    /// * `Some(token)` if this is a challenge path
    /// * `None` if this is not a challenge path
    #[inline]
    pub fn extract_token_from_path(path: &str) -> Option<&str> {
        path.strip_prefix(ACME_CHALLENGE_PATH_PREFIX)
    }

    /// Try to handle an ACME HTTP-01 challenge request.
    ///
    /// This is the main entry point for request handling. It performs:
    /// 1. Fast path check (is this a challenge path?)
    /// 2. Token extraction (no allocation)
    /// 3. Key authorization lookup (DashMap get)
    ///
    /// # Arguments
    /// * `path` - The request path
    ///
    /// # Returns
    /// * `Some(key_authorization)` if this is a valid challenge request
    /// * `None` if this is not a challenge request or the token is unknown
    #[inline]
    pub fn try_handle_challenge(&self, path: &str) -> Option<String> {
        // Fast path: check prefix first (no allocation)
        let token = Self::extract_token_from_path(path)?;

        // Token must not be empty and should not contain path separators
        if token.is_empty() || token.contains('/') {
            return None;
        }

        // Look up the key authorization
        self.get_key_authorization(token)
    }

    /// Clean up expired challenge entries.
    pub fn cleanup_expired(&self) {
        self.challenges.retain(|_, entry| !entry.is_expired());
    }

    /// Get the number of active challenges (for monitoring).
    pub fn active_challenge_count(&self) -> usize {
        self.challenges.len()
    }
}

impl Default for AcmeHttp01ChallengeStore {
    fn default() -> Self {
        Self::new()
    }
}

// Global singleton for the challenge store
static CHALLENGE_STORE: std::sync::OnceLock<Arc<AcmeHttp01ChallengeStore>> = std::sync::OnceLock::new();

/// Get the global ACME HTTP-01 challenge store.
///
/// This is a singleton that is shared across all request handlers.
/// The store is created lazily on first access.
pub fn get_tls_http01_challenge_store() -> Arc<AcmeHttp01ChallengeStore> {
    CHALLENGE_STORE.get_or_init(|| Arc::new(AcmeHttp01ChallengeStore::new())).clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_path() {
        // Valid challenge paths
        assert_eq!(AcmeHttp01ChallengeStore::extract_token_from_path("/.well-known/acme-challenge/abc123"), Some("abc123"));
        assert_eq!(
            AcmeHttp01ChallengeStore::extract_token_from_path("/.well-known/acme-challenge/token-with-dashes"),
            Some("token-with-dashes")
        );

        // Not challenge paths
        assert_eq!(AcmeHttp01ChallengeStore::extract_token_from_path("/"), None);
        assert_eq!(AcmeHttp01ChallengeStore::extract_token_from_path("/index.html"), None);
        assert_eq!(AcmeHttp01ChallengeStore::extract_token_from_path("/.well-known/other"), None);
    }

    #[test]
    fn test_challenge_store_basic() {
        let store = AcmeHttp01ChallengeStore::new();

        // Add a challenge
        store.add_challenge("token123".to_string(), "token123.thumbprint".to_string());

        // Should be able to retrieve it
        assert_eq!(store.get_key_authorization("token123"), Some("token123.thumbprint".to_string()));

        // Unknown token should return None
        assert_eq!(store.get_key_authorization("unknown"), None);

        // Remove the challenge
        store.remove_challenge("token123");
        assert_eq!(store.get_key_authorization("token123"), None);
    }

    #[test]
    fn test_try_handle_challenge() {
        let store = AcmeHttp01ChallengeStore::new();
        store.add_challenge("mytoken".to_string(), "mytoken.mythumbprint".to_string());

        // Valid challenge request
        assert_eq!(store.try_handle_challenge("/.well-known/acme-challenge/mytoken"), Some("mytoken.mythumbprint".to_string()));

        // Not a challenge path
        assert_eq!(store.try_handle_challenge("/index.html"), None);

        // Challenge path but unknown token
        assert_eq!(store.try_handle_challenge("/.well-known/acme-challenge/unknown"), None);

        // Empty token
        assert_eq!(store.try_handle_challenge("/.well-known/acme-challenge/"), None);

        // Token with path separator (invalid)
        assert_eq!(store.try_handle_challenge("/.well-known/acme-challenge/foo/bar"), None);
    }

    #[test]
    fn test_active_challenge_count() {
        let store = AcmeHttp01ChallengeStore::new();
        assert_eq!(store.active_challenge_count(), 0);

        store.add_challenge("t1".to_string(), "k1".to_string());
        assert_eq!(store.active_challenge_count(), 1);

        store.add_challenge("t2".to_string(), "k2".to_string());
        assert_eq!(store.active_challenge_count(), 2);

        store.remove_challenge("t1");
        assert_eq!(store.active_challenge_count(), 1);
    }
}
