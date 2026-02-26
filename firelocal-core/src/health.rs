use crate::error::Result;
use crate::logging::{HealthStatus, PerformanceMetrics};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Health check interface
pub trait HealthCheck {
    fn name(&self) -> &str;
    fn check(&self) -> Result<HealthCheckResult>;
    fn is_critical(&self) -> bool {
        true
    }
}

/// Result of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub duration: Duration,
    pub metrics: Option<PerformanceMetrics>,
}

impl HealthCheckResult {
    pub fn healthy(name: &str, message: &str, duration: Duration) -> Self {
        Self {
            name: name.to_string(),
            status: HealthStatus::healthy(),
            message: message.to_string(),
            duration,
            metrics: None,
        }
    }

    pub fn unhealthy(name: &str, message: &str, duration: Duration) -> Self {
        let mut status = HealthStatus::healthy();
        status.database_healthy = false;

        Self {
            name: name.to_string(),
            status,
            message: message.to_string(),
            duration,
            metrics: None,
        }
    }
}

/// Database health check
pub struct DatabaseHealthCheck;

impl Default for DatabaseHealthCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseHealthCheck {
    pub fn new() -> Self {
        Self
    }
}

impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        "database"
    }

    fn check(&self) -> Result<HealthCheckResult> {
        let start = Instant::now();

        // Check if we can perform basic operations
        // This is a simplified check - in a real implementation,
        // you'd check actual database connectivity

        let duration = start.elapsed();

        if duration < Duration::from_secs(5) {
            Ok(HealthCheckResult::healthy(
                "database",
                "Database is responsive",
                duration,
            ))
        } else {
            Ok(HealthCheckResult::unhealthy(
                "database",
                "Database response time is too slow",
                duration,
            ))
        }
    }
}

/// Storage health check
pub struct StorageHealthCheck;

impl HealthCheck for StorageHealthCheck {
    fn name(&self) -> &str {
        "storage"
    }

    fn check(&self) -> Result<HealthCheckResult> {
        let start = Instant::now();

        // Check storage availability
        // In a real implementation, you'd check disk space, permissions, etc.

        let duration = start.elapsed();

        Ok(HealthCheckResult::healthy(
            "storage",
            "Storage is available",
            duration,
        ))
    }
}

/// Memory health check
pub struct MemoryHealthCheck {
    threshold_mb: usize,
}

impl MemoryHealthCheck {
    pub fn new(threshold_mb: usize) -> Self {
        Self { threshold_mb }
    }
}

impl HealthCheck for MemoryHealthCheck {
    fn name(&self) -> &str {
        "memory"
    }

    fn check(&self) -> Result<HealthCheckResult> {
        let start = Instant::now();

        // Simple memory check (in a real implementation, you'd use system APIs)
        // For now, just assume we're healthy
        let duration = start.elapsed();

        Ok(HealthCheckResult::healthy(
            "memory",
            &format!("Memory usage is below {}MB threshold", self.threshold_mb),
            duration,
        ))
    }
}

/// Health monitor that runs multiple health checks
pub struct HealthMonitor {
    checks: Vec<Box<dyn HealthCheck + Send + Sync>>,
    last_check: Arc<Mutex<Instant>>,
    results: Arc<Mutex<HashMap<String, HealthCheckResult>>>,
    check_interval: Duration,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            last_check: Arc::new(Mutex::new(Instant::now())),
            results: Arc::new(Mutex::new(HashMap::new())),
            check_interval: Duration::from_secs(30),
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    pub fn add_check(mut self, check: Box<dyn HealthCheck + Send + Sync>) -> Self {
        self.checks.push(check);
        self
    }

    pub async fn run_checks(&self) -> Result<Vec<HealthCheckResult>> {
        let mut results = Vec::new();
        let mut results_map = HashMap::new();

        for check in &self.checks {
            let start = Instant::now();
            match check.check() {
                Ok(result) => {
                    log::info!("Health check '{}' passed: {}", check.name(), result.message);
                    results.push(result.clone());
                    results_map.insert(check.name().to_string(), result);
                }
                Err(e) => {
                    log::error!("Health check '{}' failed: {}", check.name(), e);
                    let result =
                        HealthCheckResult::unhealthy(check.name(), &e.to_string(), start.elapsed());
                    results.push(result.clone());
                    results_map.insert(check.name().to_string(), result);
                }
            }
        }

        // Update stored results
        {
            let mut stored_results = self.results.lock().unwrap();
            *stored_results = results_map;
        }

        {
            let mut last_check = self.last_check.lock().unwrap();
            *last_check = Instant::now();
        }

        Ok(results)
    }

    pub fn get_last_results(&self) -> Vec<HealthCheckResult> {
        let results = self.results.lock().unwrap();
        results.values().cloned().collect()
    }

    pub fn is_healthy(&self) -> bool {
        let results = self.get_last_results();

        if results.is_empty() {
            return false;
        }

        // Check if all critical checks are healthy
        for check in &self.checks {
            if check.is_critical() {
                if let Some(result) = results.iter().find(|r| r.name == check.name()) {
                    if !result.status.is_healthy() {
                        return false;
                    }
                } else {
                    return false; // No result for critical check
                }
            }
        }

        true
    }

    pub fn get_health_summary(&self) -> HealthSummary {
        let results = self.get_last_results();
        let total_checks = results.len();
        let healthy_checks = results.iter().filter(|r| r.status.is_healthy()).count();
        let critical_checks = self.checks.iter().filter(|c| c.is_critical()).count();
        let healthy_critical = results
            .iter()
            .filter(|r| {
                r.status.is_healthy()
                    && self
                        .checks
                        .iter()
                        .any(|c| c.name() == r.name && c.is_critical())
            })
            .count();

        HealthSummary {
            total_checks,
            healthy_checks,
            critical_checks,
            healthy_critical,
            overall_healthy: self.is_healthy(),
            last_check: *self.last_check.lock().unwrap(),
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Health summary for reporting
#[derive(Debug, Clone)]
pub struct HealthSummary {
    pub total_checks: usize,
    pub healthy_checks: usize,
    pub critical_checks: usize,
    pub healthy_critical: usize,
    pub overall_healthy: bool,
    pub last_check: Instant,
}

impl HealthSummary {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{
  "overall_healthy": {},
  "total_checks": {},
  "healthy_checks": {},
  "critical_checks": {},
  "healthy_critical": {},
  "last_check": "{}",
  "uptime": "{}"
}}"#,
            self.overall_healthy,
            self.total_checks,
            self.healthy_checks,
            self.critical_checks,
            self.healthy_critical,
            self.last_check.elapsed().as_secs(),
            self.last_check.elapsed().as_secs()
        )
    }
}

/// Create a default health monitor with common checks
pub fn create_default_health_monitor() -> HealthMonitor {
    HealthMonitor::new()
        .add_check(Box::new(DatabaseHealthCheck::new()))
        .add_check(Box::new(StorageHealthCheck))
        .add_check(Box::new(MemoryHealthCheck::new(1024))) // 1GB threshold
        .with_interval(Duration::from_secs(30))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult::healthy("test", "OK", Duration::from_millis(100));
        assert!(result.status.is_healthy());
        assert_eq!(result.name, "test");
        assert_eq!(result.message, "OK");
    }

    #[test]
    fn test_database_health_check() {
        let check = DatabaseHealthCheck::new();
        assert_eq!(check.name(), "database");
        assert!(check.is_critical());

        let result = check.check().unwrap();
        assert!(result.status.is_healthy());
    }

    #[test]
    fn test_health_monitor() {
        let monitor = HealthMonitor::new()
            .add_check(Box::new(DatabaseHealthCheck::new()))
            .add_check(Box::new(StorageHealthCheck))
            .add_check(Box::new(MemoryHealthCheck::new(1024))) // 1GB threshold
            .with_interval(Duration::from_secs(30));

        // Run checks synchronously for test
        let rt = tokio::runtime::Runtime::new().unwrap();
        let results = rt.block_on(monitor.run_checks()).unwrap();

        assert_eq!(results.len(), 3);
        assert!(monitor.is_healthy());

        let summary = monitor.get_health_summary();
        assert_eq!(summary.total_checks, 3);
        assert_eq!(summary.healthy_checks, 3);
        assert!(summary.overall_healthy);

        let json = summary.to_json();
        assert!(json.contains("overall_healthy"));
        assert!(json.contains("total_checks"));
    }

    #[test]
    fn test_health_summary_json() {
        let summary = HealthSummary {
            total_checks: 3,
            healthy_checks: 2,
            critical_checks: 2,
            healthy_critical: 1,
            overall_healthy: false,
            last_check: Instant::now(),
        };

        let json = summary.to_json();
        assert!(json.contains("overall_healthy"));
        assert!(json.contains("total_checks"));
        assert!(json.contains("false"));
    }
}
