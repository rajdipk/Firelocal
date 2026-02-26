use crate::error::{FireLocalError, Result};
use crate::logging::log_security_event;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable authentication
    pub authentication_enabled: bool,
    /// Enable authorization checks
    pub authorization_enabled: bool,
    /// Enable rate limiting
    pub rate_limit_enabled: bool,
    /// Enable audit logging
    pub audit_logging_enabled: bool,
    /// Maximum requests per minute per client
    pub max_requests_per_minute: u32,
    /// Maximum document size in bytes
    pub max_document_size: usize,
    /// Maximum path depth
    pub max_path_depth: usize,
    /// Blocked IP addresses
    pub blocked_ips: Vec<String>,
    /// Allowed operations for anonymous users
    pub anonymous_operations: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            authentication_enabled: false,
            authorization_enabled: true,
            rate_limit_enabled: true,
            audit_logging_enabled: true,
            max_requests_per_minute: 1000,
            max_document_size: 10 * 1024 * 1024, // 10MB
            max_path_depth: 32,
            blocked_ips: Vec::new(),
            anonymous_operations: vec!["read".to_string()],
        }
    }
}

/// Security context for operations
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// User ID if authenticated
    pub user_id: Option<String>,
    /// User roles
    pub roles: Vec<String>,
    /// Client IP address
    pub client_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Request timestamp
    pub timestamp: Instant,
}

impl SecurityContext {
    pub fn anonymous() -> Self {
        Self {
            user_id: None,
            roles: Vec::new(),
            client_ip: None,
            user_agent: None,
            auth_token: None,
            timestamp: Instant::now(),
        }
    }

    pub fn authenticated(user_id: &str, roles: Vec<String>) -> Self {
        Self {
            user_id: Some(user_id.to_string()),
            roles,
            client_ip: None,
            user_agent: None,
            auth_token: None,
            timestamp: Instant::now(),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

/// Rate limiter for security
pub struct SecurityRateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: u32,
    window: Duration,
}

impl SecurityRateLimiter {
    pub fn new(max_requests: u32, window_minutes: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_minutes * 60),
        }
    }

    pub fn check_rate_limit(&self, client_id: &str) -> Result<()> {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        let client_requests = requests.entry(client_id.to_string()).or_default();

        // Remove old requests outside the window
        client_requests.retain(|&time| now.duration_since(time) < self.window);

        if client_requests.len() >= self.max_requests as usize {
            log_security_event(
                "RATE_LIMIT_EXCEEDED",
                &format!("Client {} exceeded rate limit", client_id),
            );
            return Err(FireLocalError::RateLimitExceeded(format!(
                "Rate limit exceeded: {} requests per {:?}",
                self.max_requests, self.window
            )));
        }

        client_requests.push(now);
        Ok(())
    }
}

/// Input sanitizer
pub struct InputSanitizer;

impl InputSanitizer {
    /// Sanitize document path
    pub fn sanitize_path(path: &str, max_depth: usize) -> Result<String> {
        if path.is_empty() {
            return Err(FireLocalError::Validation(
                "Path cannot be empty".to_string(),
            ));
        }

        if path.len() > 4096 {
            return Err(FireLocalError::Validation("Path too long".to_string()));
        }

        // Remove leading/trailing slashes and whitespace
        let path = path.trim().trim_matches('/');

        // Check for path traversal attempts
        if path.contains("..") || path.contains("\\") {
            log_security_event("PATH_TRAVERSAL_ATTEMPT", &format!("Path: {}", path));
            return Err(FireLocalError::Security(
                "Path traversal detected".to_string(),
            ));
        }

        // Check for null bytes
        if path.contains('\0') {
            log_security_event("NULL_BYTE_INJECTION", &format!("Path: {}", path));
            return Err(FireLocalError::Security(
                "Null byte detected in path".to_string(),
            ));
        }

        // Check depth
        let depth = path.split('/').count();
        if depth > max_depth {
            return Err(FireLocalError::Validation(format!(
                "Path depth {} exceeds maximum {}",
                depth, max_depth
            )));
        }

        // Validate characters
        for segment in path.split('/') {
            if segment.is_empty() {
                return Err(FireLocalError::Validation("Empty path segment".to_string()));
            }

            for c in segment.chars() {
                if !c.is_alphanumeric() && !"-_.".contains(c) {
                    return Err(FireLocalError::Validation(format!(
                        "Invalid character '{}' in path",
                        c
                    )));
                }
            }
        }

        Ok(path.to_string())
    }

    /// Sanitize document data
    pub fn sanitize_document(data: &[u8], max_size: usize) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(FireLocalError::Validation(
                "Document cannot be empty".to_string(),
            ));
        }

        if data.len() > max_size {
            return Err(FireLocalError::Validation(format!(
                "Document size {} exceeds maximum {}",
                data.len(),
                max_size
            )));
        }

        // Check for null bytes
        if data.contains(&0) {
            log_security_event("NULL_BYTE_IN_DOCUMENT", "Document contains null bytes");
            return Err(FireLocalError::Security(
                "Null byte detected in document".to_string(),
            ));
        }

        // Validate UTF-8
        match std::str::from_utf8(data) {
            Ok(_) => Ok(data.to_vec()),
            Err(_) => Err(FireLocalError::Validation(
                "Document must be valid UTF-8".to_string(),
            )),
        }
    }

    /// Validate JSON structure
    pub fn validate_json(data: &[u8]) -> Result<()> {
        let json_str = std::str::from_utf8(data)
            .map_err(|_| FireLocalError::Validation("Invalid UTF-8".to_string()))?;

        // Check for potentially dangerous JSON patterns
        if json_str.contains("__proto__") || json_str.contains("constructor") {
            log_security_event(
                "PROTOTYPE_POLLUTION",
                "JSON contains prototype pollution patterns",
            );
            return Err(FireLocalError::Security(
                "Prototype pollution detected".to_string(),
            ));
        }

        // Try to parse as JSON
        serde_json::from_str::<serde_json::Value>(json_str)
            .map_err(|e| FireLocalError::Validation(format!("Invalid JSON: {}", e)))?;

        Ok(())
    }
}

/// Security auditor for logging and monitoring
pub struct SecurityAuditor {
    config: SecurityConfig,
    rate_limiter: SecurityRateLimiter,
}

impl SecurityAuditor {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            rate_limiter: SecurityRateLimiter::new(config.max_requests_per_minute, 1),
            config,
        }
    }

    /// Perform security checks before an operation
    pub fn pre_operation_check(
        &self,
        context: &SecurityContext,
        operation: &str,
        path: &str,
        data: Option<&[u8]>,
    ) -> Result<()> {
        // Check IP blocking
        if let Some(ip) = &context.client_ip {
            if self.config.blocked_ips.contains(ip) {
                log_security_event(
                    "BLOCKED_IP_ACCESS",
                    &format!("IP: {}, Operation: {}", ip, operation),
                );
                return Err(FireLocalError::PermissionDenied(
                    "IP address is blocked".to_string(),
                ));
            }
        }

        // Check rate limiting
        if self.config.rate_limit_enabled {
            let client_id = context
                .client_ip
                .clone()
                .unwrap_or_else(|| "anonymous".to_string());

            self.rate_limiter.check_rate_limit(&client_id)?;
        }

        // Check authentication
        if self.config.authentication_enabled
            && !context.is_authenticated()
            && !self
                .config
                .anonymous_operations
                .contains(&operation.to_string())
        {
            log_security_event(
                "UNAUTHORIZED_ACCESS_ATTEMPT",
                &format!("Operation: {}, Path: {}", operation, path),
            );
            return Err(FireLocalError::PermissionDenied(
                "Authentication required".to_string(),
            ));
        }

        // Validate and sanitize inputs
        let sanitized_path = InputSanitizer::sanitize_path(path, self.config.max_path_depth)?;

        if let Some(data) = data {
            InputSanitizer::sanitize_document(data, self.config.max_document_size)?;
            InputSanitizer::validate_json(data)?;
        }

        // Log the operation if audit logging is enabled
        if self.config.audit_logging_enabled {
            self.log_operation(context, operation, &sanitized_path, data.is_some());
        }

        Ok(())
    }

    /// Log security events
    fn log_operation(
        &self,
        context: &SecurityContext,
        operation: &str,
        path: &str,
        has_data: bool,
    ) {
        let user_info = context
            .user_id
            .clone()
            .unwrap_or_else(|| "anonymous".to_string());

        let ip_info = context
            .client_ip
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        log_security_event(
            "OPERATION",
            &format!(
                "User: {}, IP: {}, Operation: {}, Path: {}, HasData: {}",
                user_info, ip_info, operation, path, has_data
            ),
        );
    }

    /// Check if user has permission for operation
    pub fn check_permission(
        &self,
        context: &SecurityContext,
        operation: &str,
        resource: &str,
    ) -> Result<()> {
        if !self.config.authorization_enabled {
            return Ok(());
        }

        // Admin users have all permissions
        if context.has_role("admin") {
            return Ok(());
        }

        // Check role-based permissions
        match operation {
            "read" => {
                if context.has_role("reader") || context.has_role("writer") {
                    return Ok(());
                }
            }
            "write" | "delete" => {
                if context.has_role("writer") {
                    return Ok(());
                }
            }
            _ => {}
        }

        log_security_event(
            "PERMISSION_DENIED",
            &format!(
                "User: {}, Operation: {}, Resource: {}",
                context.user_id.as_ref().unwrap_or(&"anonymous".to_string()),
                operation,
                resource
            ),
        );

        Err(FireLocalError::PermissionDenied(format!(
            "Insufficient permissions for operation '{}' on resource '{}'",
            operation, resource
        )))
    }
}

/// Create default security auditor
pub fn create_default_security_auditor() -> SecurityAuditor {
    SecurityAuditor::new(SecurityConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_sanitizer_path() {
        // Valid path
        assert!(InputSanitizer::sanitize_path("users/alice", 32).is_ok());

        // Path traversal
        assert!(InputSanitizer::sanitize_path("../../../etc/passwd", 32).is_err());

        // Too deep
        let deep_path = "a/".repeat(40);
        assert!(InputSanitizer::sanitize_path(&deep_path, 32).is_err());

        // Invalid characters
        assert!(InputSanitizer::sanitize_path("users/alice@domain", 32).is_err());
    }

    #[test]
    fn test_input_sanitizer_document() {
        // Valid document
        let valid_doc = br#"{"name": "Alice", "age": 30}"#;
        assert!(InputSanitizer::sanitize_document(valid_doc, 1024).is_ok());

        // Too large
        let large_doc = vec![b'x'; 1024 * 1024 + 1];
        assert!(InputSanitizer::sanitize_document(&large_doc, 1024 * 1024).is_err());

        // Null bytes
        let null_doc = b"test\x00data";
        assert!(InputSanitizer::sanitize_document(null_doc, 1024).is_err());
    }

    #[test]
    fn test_security_context() {
        let anonymous = SecurityContext::anonymous();
        assert!(!anonymous.is_authenticated());

        let authenticated = SecurityContext::authenticated("user123", vec!["reader".to_string()]);
        assert!(authenticated.is_authenticated());
        assert!(authenticated.has_role("reader"));
        assert!(!authenticated.has_role("admin"));
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = SecurityRateLimiter::new(2, 1); // 2 requests per minute

        assert!(limiter.check_rate_limit("client1").is_ok());
        assert!(limiter.check_rate_limit("client1").is_ok());
        assert!(limiter.check_rate_limit("client1").is_err());
    }

    #[test]
    fn test_security_auditor() {
        let auditor = create_default_security_auditor();
        let context = SecurityContext::anonymous();

        // Should succeed for anonymous read (in default config)
        assert!(auditor
            .pre_operation_check(&context, "read", "users/alice", None)
            .is_ok());

        // Should also succeed for anonymous write (authentication disabled by default)
        assert!(auditor
            .pre_operation_check(&context, "write", "users/alice", None)
            .is_ok());

        // Test with authentication enabled
        let auth_auditor = SecurityAuditor::new(SecurityConfig {
            authentication_enabled: true,
            ..Default::default()
        });

        // Should succeed for read
        assert!(auth_auditor
            .pre_operation_check(&context, "read", "users/alice", None)
            .is_ok());

        // Should fail for write when authentication is enabled
        assert!(auth_auditor
            .pre_operation_check(&context, "write", "users/alice", None)
            .is_err());
    }
}
