use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use chrono::{DateTime, Utc};

/// Application-level metrics collected in-process.
/// No third-party monitoring — all queryable via the /metrics endpoint.
pub struct AppMetrics {
    pub start_time: DateTime<Utc>,
    pub total_requests: AtomicU64,
    pub total_errors: AtomicU64,
    pub active_connections: AtomicU64,
}

static METRICS: OnceLock<AppMetrics> = OnceLock::new();

pub fn init() -> &'static AppMetrics {
    METRICS.get_or_init(|| AppMetrics {
        start_time: Utc::now(),
        total_requests: AtomicU64::new(0),
        total_errors: AtomicU64::new(0),
        active_connections: AtomicU64::new(0),
    })
}

pub fn get() -> &'static AppMetrics {
    METRICS.get().expect("Metrics not initialized")
}

impl AppMetrics {
    pub fn inc_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_errors(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let now = Utc::now();
        let uptime = now.signed_duration_since(self.start_time);
        MetricsSnapshot {
            uptime_seconds: uptime.num_seconds(),
            start_time: self.start_time,
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
        }
    }
}

#[derive(serde::Serialize)]
pub struct MetricsSnapshot {
    pub uptime_seconds: i64,
    pub start_time: DateTime<Utc>,
    pub total_requests: u64,
    pub total_errors: u64,
    pub active_connections: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metrics() -> AppMetrics {
        AppMetrics {
            start_time: Utc::now(),
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
        }
    }

    #[test]
    fn test_inc_requests() {
        let m = make_metrics();
        m.inc_requests();
        m.inc_requests();
        m.inc_requests();
        assert_eq!(m.total_requests.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_inc_errors() {
        let m = make_metrics();
        m.inc_errors();
        assert_eq!(m.total_errors.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_connections_inc_dec() {
        let m = make_metrics();
        m.inc_connections();
        m.inc_connections();
        assert_eq!(m.active_connections.load(Ordering::Relaxed), 2);
        m.dec_connections();
        assert_eq!(m.active_connections.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_snapshot() {
        let m = make_metrics();
        m.inc_requests();
        m.inc_requests();
        m.inc_errors();
        let snap = m.snapshot();
        assert_eq!(snap.total_requests, 2);
        assert_eq!(snap.total_errors, 1);
        assert_eq!(snap.active_connections, 0);
        assert!(snap.uptime_seconds >= 0);
    }

    #[test]
    fn test_init_returns_same_instance() {
        let m1 = init();
        let m2 = init();
        // Both should point to the same static
        assert!(std::ptr::eq(m1, m2));
    }
}
