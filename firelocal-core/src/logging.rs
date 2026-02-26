use log::{error, info, warn};
use std::time::{Duration, Instant};

/// Performance metrics for operations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation_count: u64,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub errors: u64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            operation_count: 0,
            total_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            errors: 0,
        }
    }

    pub fn record_operation(&mut self, duration: Duration, success: bool) {
        self.operation_count += 1;
        self.total_duration += duration;

        if duration < self.min_duration {
            self.min_duration = duration;
        }
        if duration > self.max_duration {
            self.max_duration = duration;
        }

        if !success {
            self.errors += 1;
        }

        if self.operation_count > 0 {
            self.avg_duration = self.total_duration / self.operation_count as u32;
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.operation_count == 0 {
            1.0
        } else {
            (self.operation_count - self.errors) as f64 / self.operation_count as f64
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for timing operations with logging
#[macro_export]
macro_rules! timed_operation {
    ($operation_type:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();

        match &result {
            Ok(_) => {
                info!("{} completed in {:?}", $operation_type, duration);
                debug!("{} success - duration: {:?}", $operation_type, duration);
            }
            Err(e) => {
                error!("{} failed in {:?}: {}", $operation_type, duration, e);
                warn!(
                    "{} error - duration: {:?}, error: {}",
                    $operation_type, duration, e
                );
            }
        }

        (result, duration)
    }};
}

/// Health check status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub database_healthy: bool,
    pub storage_healthy: bool,
    pub last_check: Instant,
    pub uptime: Duration,
    pub metrics: PerformanceMetrics,
}

impl HealthStatus {
    pub fn healthy() -> Self {
        Self {
            database_healthy: true,
            storage_healthy: true,
            last_check: Instant::now(),
            uptime: Duration::ZERO,
            metrics: PerformanceMetrics::new(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.database_healthy && self.storage_healthy
    }

    pub fn update_check(&mut self) {
        self.last_check = Instant::now();
    }
}

/// Configuration for logging
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size: u64,
    pub max_files: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_path: None,
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
        }
    }
}

/// Initialize logging with the given configuration
pub fn init_logging(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::OpenOptions;

    let level = match config.level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    };

    if let Some(file_path) = &config.file_path {
        // Simple file logging setup
        let _file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        // In a real implementation, you'd use a proper logging crate like fern or env_logger
        println!("Logging to file: {} at level: {:?}", file_path, level);
    } else {
        // Console logging
        println!("Console logging at level: {:?}", level);
    }

    // For now, just set up basic logging
    // In production, you'd use a proper logging framework
    Ok(())
}

/// Log database operations
pub fn log_database_operation(operation: &str, path: &str, success: bool, duration: Duration) {
    if success {
        info!(
            "DB OP: {} on {} succeeded in {:?}",
            operation, path, duration
        );
    } else {
        error!("DB OP: {} on {} failed in {:?}", operation, path, duration);
    }
}

/// Log security events
pub fn log_security_event(event: &str, details: &str) {
    warn!("SECURITY: {} - {}", event, details);
}

/// Log performance metrics
pub fn log_performance_metrics(metrics: &PerformanceMetrics, operation_type: &str) {
    info!(
        "PERFORMANCE {}: count={}, avg={:?}, min={:?}, max={:?}, success_rate={:.2}%",
        operation_type,
        metrics.operation_count,
        metrics.avg_duration,
        metrics.min_duration,
        metrics.max_duration,
        metrics.success_rate() * 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_operation(Duration::from_millis(100), true);
        metrics.record_operation(Duration::from_millis(200), true);
        metrics.record_operation(Duration::from_millis(50), false);

        assert_eq!(metrics.operation_count, 3);
        assert_eq!(metrics.errors, 1);
        assert!(metrics.success_rate() > 0.5);
        assert!(metrics.success_rate() < 1.0);
    }

    #[test]
    fn test_health_status() {
        let mut status = HealthStatus::healthy();
        assert!(status.is_healthy());

        status.database_healthy = false;
        assert!(!status.is_healthy());
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.max_file_size, 10 * 1024 * 1024);
        assert_eq!(config.max_files, 5);
    }
}
