/// Input validation module for security and data integrity
use anyhow::{anyhow, Result};

/// Validate document path
///
/// Only basic validation:
/// - Not empty
/// - Maximum 4096 characters
const MAX_PATH_LENGTH: usize = 4096;
const MAX_PATH_DEPTH: usize = 100;

/// Validate document path
///
/// Rules:
/// - Not empty
/// - Maximum 4096 characters
/// - Maximum 100 segments
/// - No leading/trailing slashes
/// - No consecutive slashes
/// - Valid characters: alphanumeric, underscore, hyphen
pub fn validate_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(anyhow!("Path cannot be empty"));
    }

    if path.len() > MAX_PATH_LENGTH {
        return Err(anyhow!(
            "Path too long (max {} characters)",
            MAX_PATH_LENGTH
        ));
    }

    if path.starts_with('/') || path.ends_with('/') {
        return Err(anyhow!("Path cannot start or end with '/'"));
    }

    if path.contains("//") {
        return Err(anyhow!("Path cannot contain consecutive slashes"));
    }

    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() > MAX_PATH_DEPTH {
        return Err(anyhow!("Path too deep (max {} segments)", MAX_PATH_DEPTH));
    }

    for segment in segments {
        if segment.is_empty() {
            return Err(anyhow!("Path segment cannot be empty"));
        }
        for c in segment.chars() {
            if !c.is_alphanumeric() && c != '_' && c != '-' {
                return Err(anyhow!("Invalid character in path: '{}'", c));
            }
        }
    }

    Ok(())
}

/// Validate document data size
///
/// Rules:
/// - Not empty
/// - Maximum 100MB per document
pub fn validate_data_size(data: &[u8]) -> Result<()> {
    const MAX_SIZE: usize = 100 * 1024 * 1024; // 100MB

    if data.is_empty() {
        return Err(anyhow!("Document data cannot be empty"));
    }

    if data.len() > MAX_SIZE {
        return Err(anyhow!(
            "Document too large ({} MB, max 100 MB)",
            data.len() / 1024 / 1024
        ));
    }

    Ok(())
}

/// Validate JSON data
///
/// Rules:
/// - Valid UTF-8
/// - Valid JSON structure
pub fn validate_json(data: &[u8]) -> Result<()> {
    // Check if valid UTF-8
    let s = std::str::from_utf8(data).map_err(|_| anyhow!("Document data must be valid UTF-8"))?;

    // Strict JSON validation
    serde_json::from_str::<serde::de::IgnoredAny>(s).map_err(|_| anyhow!("Invalid JSON data"))?;

    Ok(())
}

/// Validate security rules
///
/// Simplified validation:
/// - Not empty
/// - Maximum 1MB
/// - Must contain "service cloud.firestore"
pub fn validate_rules(rules: &str) -> Result<()> {
    if rules.is_empty() {
        return Err(anyhow!("Rules cannot be empty"));
    }

    if rules.len() > 1024 * 1024 {
        return Err(anyhow!("Rules too large (max 1MB)"));
    }

    if !rules.contains("service cloud.firestore") {
        return Err(anyhow!(
            "Invalid rules format: missing 'service cloud.firestore'"
        ));
    }

    Ok(())
}

/// Rate limiter for operations
pub struct RateLimiter {
    max_requests: usize,
    window_secs: u64,
    requests: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<std::time::Instant>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_requests` - Maximum requests per window
    /// * `window_secs` - Time window in seconds
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
            requests: std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new())),
        }
    }

    /// Check if operation is allowed
    pub fn check(&self) -> Result<()> {
        let mut requests = self
            .requests
            .lock()
            .map_err(|_| anyhow!("Rate limiter lock poisoned"))?;

        let now = std::time::Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        // Remove old requests outside the window
        while let Some(&req_time) = requests.front() {
            if now.duration_since(req_time) > window {
                requests.pop_front();
            } else {
                break;
            }
        }

        // Check if limit exceeded
        if requests.len() >= self.max_requests {
            return Err(anyhow!(
                "Rate limit exceeded: {} requests per {} seconds",
                self.max_requests,
                self.window_secs
            ));
        }

        // Record this request
        requests.push_back(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_valid() {
        assert!(validate_path("users/alice").is_ok());
        assert!(validate_path("users/alice/posts/post1").is_ok());
        assert!(validate_path("users-2024/alice_123").is_ok());
    }

    #[test]
    fn test_validate_path_invalid() {
        assert!(validate_path("").is_err());
        assert!(validate_path("/users/alice").is_err());
        assert!(validate_path("users/alice/").is_err());
        assert!(validate_path("users//alice").is_err());
        assert!(validate_path("users/alice@domain").is_err());
    }

    #[test]
    fn test_validate_path_too_long() {
        let long_path = "a/".repeat(600);
        assert!(validate_path(&long_path).is_err());
    }

    #[test]
    fn test_validate_data_size() {
        assert!(validate_data_size(b"test").is_ok());
        assert!(validate_data_size(b"").is_err());
    }

    #[test]
    fn test_validate_json_valid() {
        assert!(validate_json(br#"{"name":"Alice"}"#).is_ok());
        assert!(validate_json(br#"[1,2,3]"#).is_ok());
        assert!(validate_json(br#""string""#).is_ok());
    }

    #[test]
    fn test_validate_json_invalid() {
        assert!(validate_json(b"not json").is_err());
        assert!(validate_json(b"{invalid}").is_err());
    }

    #[test]
    fn test_validate_rules() {
        let valid_rules = r#"
            service cloud.firestore {
                match /databases/{database}/documents {
                    match /{document=**} {
                        allow read, write: if true;
                    }
                }
            }
        "#;
        assert!(validate_rules(valid_rules).is_ok());

        assert!(validate_rules("").is_err());
        assert!(validate_rules("invalid rules").is_err());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, 1);

        // First 3 requests should succeed
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());

        // 4th request should fail
        assert!(limiter.check().is_err());
    }
}
