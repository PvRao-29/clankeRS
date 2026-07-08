//! Integration tests for the runtime builder and metrics counters.

use std::sync::atomic::Ordering;
use std::time::Duration;

use clankers_runtime::{RobotRuntime, RuntimeMetrics};

#[test]
fn builder_defaults() {
    let rt = RobotRuntime::builder().build().unwrap();
    assert_eq!(rt.max_queue_depth, 4);
    assert_eq!(rt.deadline, None);
    assert!(!rt.latency_tracing);
}

#[test]
fn builder_overrides() {
    let rt = RobotRuntime::builder()
        .max_queue_depth(16)
        .deadline(Duration::from_millis(50))
        .enable_latency_tracing(true)
        .build()
        .unwrap();
    assert_eq!(rt.max_queue_depth, 16);
    assert_eq!(rt.deadline, Some(Duration::from_millis(50)));
    assert!(rt.latency_tracing);
}

#[test]
fn metrics_counters_accumulate() {
    let metrics = RuntimeMetrics::new();
    metrics.record_dropped(3);
    metrics.record_dropped(2);
    metrics.record_deadline_miss();
    metrics.record_deadline_miss();

    assert_eq!(metrics.dropped_messages.load(Ordering::Relaxed), 5);
    assert_eq!(metrics.deadline_misses.load(Ordering::Relaxed), 2);
    assert!(!metrics.format_report().is_empty());
}

#[test]
fn runtime_exposes_metrics() {
    let rt = RobotRuntime::builder().build().unwrap();
    rt.metrics().record_dropped(1);
    assert_eq!(rt.metrics().dropped_messages.load(Ordering::Relaxed), 1);
}
