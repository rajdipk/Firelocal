use std::fmt;
use std::io;

/// Custom error types for FireLocal
#[derive(Debug, Clone)]
pub enum FireLocalError {
    /// I/O related errors
    Io(String),
    /// Validation errors
    Validation(String),
    /// Storage related errors
    Storage(String),
    /// Transaction related errors
    Transaction(String),
    /// Security/Rule errors
    Security(String),
    /// Configuration errors
    Configuration(String),
    /// Serialization/Deserialization errors
    Serialization(String),
    /// Network/Sync errors
    Network(String),
    /// Resource not found
    NotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Rate limit exceeded
    RateLimitExceeded(String),
    /// Database corruption
    Corruption(String),
    /// Generic errors
    Generic(String),
}

impl fmt::Display for FireLocalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FireLocalError::Io(msg) => write!(f, "I/O Error: {}", msg),
            FireLocalError::Validation(msg) => write!(f, "Validation Error: {}", msg),
            FireLocalError::Storage(msg) => write!(f, "Storage Error: {}", msg),
            FireLocalError::Transaction(msg) => write!(f, "Transaction Error: {}", msg),
            FireLocalError::Security(msg) => write!(f, "Security Error: {}", msg),
            FireLocalError::Configuration(msg) => write!(f, "Configuration Error: {}", msg),
            FireLocalError::Serialization(msg) => write!(f, "Serialization Error: {}", msg),
            FireLocalError::Network(msg) => write!(f, "Network Error: {}", msg),
            FireLocalError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            FireLocalError::PermissionDenied(msg) => write!(f, "Permission Denied: {}", msg),
            FireLocalError::RateLimitExceeded(msg) => write!(f, "Rate Limit Exceeded: {}", msg),
            FireLocalError::Corruption(msg) => write!(f, "Database Corruption: {}", msg),
            FireLocalError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for FireLocalError {}

impl From<io::Error> for FireLocalError {
    fn from(err: io::Error) -> Self {
        FireLocalError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for FireLocalError {
    fn from(err: serde_json::Error) -> Self {
        FireLocalError::Serialization(format!("JSON error: {}", err))
    }
}

impl From<FireLocalError> for io::Error {
    fn from(err: FireLocalError) -> Self {
        match err {
            FireLocalError::Io(msg) => io::Error::other(msg),
            FireLocalError::NotFound(msg) => io::Error::new(io::ErrorKind::NotFound, msg),
            FireLocalError::PermissionDenied(msg) => {
                io::Error::new(io::ErrorKind::PermissionDenied, msg)
            }
            FireLocalError::Validation(msg) => io::Error::new(io::ErrorKind::InvalidInput, msg),
            FireLocalError::Corruption(msg) => io::Error::new(io::ErrorKind::InvalidData, msg),
            FireLocalError::RateLimitExceeded(msg) => io::Error::new(io::ErrorKind::WouldBlock, msg),
            FireLocalError::Network(msg) => io::Error::new(io::ErrorKind::ConnectionAborted, msg),
            FireLocalError::Storage(msg) => io::Error::new(io::ErrorKind::Other, msg),
            FireLocalError::Transaction(msg) => io::Error::new(io::ErrorKind::Other, msg),
            FireLocalError::Security(msg) => io::Error::new(io::ErrorKind::PermissionDenied, msg),
            FireLocalError::Configuration(msg) => io::Error::new(io::ErrorKind::InvalidInput, msg),
            FireLocalError::Serialization(msg) => io::Error::new(io::ErrorKind::InvalidData, msg),
            FireLocalError::Generic(msg) => io::Error::other(msg),
        }
    }
}

/// Result type alias for FireLocal operations
pub type Result<T> = std::result::Result<T, FireLocalError>;

/// Error context builder for better error messages
#[derive(Debug, Clone)]
pub struct ErrorContext {
    operation: String,
    path: Option<String>,
    details: Vec<String>,
}

impl ErrorContext {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            path: None,
            details: Vec::new(),
        }
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    pub fn with_detail(mut self, detail: &str) -> Self {
        self.details.push(detail.to_string());
        self
    }

    pub fn build(self, error: FireLocalError) -> FireLocalError {
        let mut message = format!("{}: {}", self.operation, error);

        if let Some(path) = &self.path {
            message = format!("{} (path: {})", message, path);
        }

        if !self.details.is_empty() {
            message = format!("{} - {}", message, self.details.join(", "));
        }

        match error {
            FireLocalError::Io(_) => FireLocalError::Io(message),
            FireLocalError::Validation(_) => FireLocalError::Validation(message),
            FireLocalError::Storage(_) => FireLocalError::Storage(message),
            FireLocalError::Transaction(_) => FireLocalError::Transaction(message),
            FireLocalError::Security(_) => FireLocalError::Security(message),
            FireLocalError::Configuration(_) => FireLocalError::Configuration(message),
            FireLocalError::Serialization(_) => FireLocalError::Serialization(message),
            FireLocalError::Network(_) => FireLocalError::Network(message),
            FireLocalError::NotFound(_) => FireLocalError::NotFound(message),
            FireLocalError::PermissionDenied(_) => FireLocalError::PermissionDenied(message),
            FireLocalError::RateLimitExceeded(_) => FireLocalError::RateLimitExceeded(message),
            FireLocalError::Corruption(_) => FireLocalError::Corruption(message),
            FireLocalError::Generic(_) => FireLocalError::Generic(message),
        }
    }
}

/// Macro for adding context to errors
#[macro_export]
macro_rules! error_context {
    ($operation:expr, $path:expr, $error:expr) => {
        ErrorContext::new($operation).with_path($path).build($error)
    };
    ($operation:expr, $error:expr) => {
        ErrorContext::new($operation).build($error)
    };
}

/// Recovery strategies for different error types
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry { max_attempts: u32, delay_ms: u64 },
    /// Fallback to alternative method
    Fallback(String),
    /// Skip the operation and continue
    Skip,
    /// Abort the entire operation
    Abort,
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        RecoveryStrategy::Retry {
            max_attempts: 3,
            delay_ms: 100,
        }
    }
}

/// Get recovery strategy for error type
pub fn get_recovery_strategy(error: &FireLocalError) -> RecoveryStrategy {
    match error {
        FireLocalError::Io(_) => RecoveryStrategy::Retry {
            max_attempts: 3,
            delay_ms: 100,
        },
        FireLocalError::Network(_) => RecoveryStrategy::Retry {
            max_attempts: 5,
            delay_ms: 1000,
        },
        FireLocalError::RateLimitExceeded(_) => RecoveryStrategy::Retry {
            max_attempts: 2,
            delay_ms: 5000,
        },
        FireLocalError::Validation(_) => RecoveryStrategy::Abort,
        FireLocalError::PermissionDenied(_) => RecoveryStrategy::Abort,
        FireLocalError::NotFound(_) => RecoveryStrategy::Skip,
        FireLocalError::Corruption(_) => RecoveryStrategy::Abort,
        _ => RecoveryStrategy::default(),
    }
}

/// Retry an operation with exponential backoff
pub fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_attempts: u32,
    initial_delay_ms: u64,
) -> std::result::Result<T, E>
where
    F: FnMut() -> std::result::Result<T, E>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay_ms;

    for attempt in 1..=max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_attempts {
                    return Err(e);
                }

                log::warn!(
                    "Operation failed (attempt {}/{}): {}, retrying in {}ms",
                    attempt,
                    max_attempts,
                    e,
                    delay
                );
                std::thread::sleep(std::time::Duration::from_millis(delay));
                delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context() {
        let error = FireLocalError::Validation("Invalid path".to_string());
        let context = ErrorContext::new("put")
            .with_path("users/invalid")
            .with_detail("contains special characters")
            .build(error);

        assert!(matches!(context, FireLocalError::Validation(_)));
        assert!(context.to_string().contains("put"));
        assert!(context.to_string().contains("users/invalid"));
    }

    #[test]
    fn test_error_conversions() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let firelocal_error: FireLocalError = io_error.into();

        assert!(matches!(firelocal_error, FireLocalError::Io(_)));
    }

    #[test]
    fn test_recovery_strategy() {
        let io_error = FireLocalError::Io("disk error".to_string());
        let strategy = get_recovery_strategy(&io_error);

        match strategy {
            RecoveryStrategy::Retry {
                max_attempts,
                delay_ms,
            } => {
                assert_eq!(max_attempts, 3);
                assert_eq!(delay_ms, 100);
            }
            _ => panic!("Expected retry strategy"),
        }

        let validation_error = FireLocalError::Validation("invalid data".to_string());
        let strategy = get_recovery_strategy(&validation_error);

        assert!(matches!(strategy, RecoveryStrategy::Abort));
    }

    #[test]
    fn test_retry_with_backoff() {
        let mut attempts = 0;
        let result = retry_with_backoff(
            || {
                attempts += 1;
                if attempts < 3 {
                    Err("temporary failure")
                } else {
                    Ok("success")
                }
            },
            3,
            10,
        );

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 3);
    }
}
