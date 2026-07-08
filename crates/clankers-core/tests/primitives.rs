//! Integration tests for the core primitives: timestamps, names, rates,
//! deadlines, latency stats, and error conversions.

use std::time::Duration;

use clankers_core::types::Deadline;
use clankers_core::{FrameId, LatencyStats, NodeName, Rate, RobotError, Timestamp, TopicName};

#[test]
fn timestamp_nanos_round_trip() {
    let ts = Timestamp::from_nanos(1_500_000_042);
    assert_eq!(ts.as_nanos(), 1_500_000_042);
    assert_eq!(ts.as_duration(), Duration::from_nanos(1_500_000_042));
}

#[test]
fn timestamp_ordering_and_now() {
    let a = Timestamp::from_nanos(10);
    let b = Timestamp::from_nanos(20);
    assert!(a < b);
    assert!(Timestamp::now().as_nanos() > 0);
}

#[test]
fn timestamp_to_datetime_is_some_for_real_time() {
    // A recent-ish nanos value converts to a UTC datetime.
    let ts = Timestamp::from_nanos(1_700_000_000_000_000_000);
    assert!(ts.to_datetime().is_some());
}

#[test]
fn topic_name_requires_leading_slash() {
    assert!(TopicName::new("/camera/image_raw").is_ok());
    assert!(TopicName::new("camera/image_raw").is_err());

    let topic = TopicName::new("/detections").unwrap();
    assert_eq!(topic.as_str(), "/detections");
    assert_eq!(topic.to_string(), "/detections");
}

#[test]
fn topic_name_from_str() {
    let topic: TopicName = "/scan".parse().unwrap();
    assert_eq!(topic.as_str(), "/scan");
    assert!("bad".parse::<TopicName>().is_err());
}

#[test]
fn node_and_frame_names() {
    assert_eq!(NodeName::new("perception").as_str(), "perception");
    assert_eq!(FrameId::new("camera_optical").as_str(), "camera_optical");
}

#[test]
fn rate_period_and_non_positive() {
    assert_eq!(Rate::new(10.0).period(), Duration::from_secs_f64(0.1));
    // Non-positive rates never fire.
    assert_eq!(Rate::new(0.0).period(), Duration::MAX);
    assert_eq!(Rate::new(-5.0).period(), Duration::MAX);
}

#[test]
fn deadline_from_millis() {
    assert_eq!(Deadline::from_millis(20).0, Duration::from_millis(20));
}

#[test]
fn latency_stats_empty_returns_none() {
    let stats = LatencyStats::new();
    assert_eq!(stats.count(), 0);
    assert!(stats.p50().is_none());
    assert!(stats.mean().is_none());
    assert_eq!(stats.format_report(), "Latency: (no samples)");
}

#[test]
fn latency_stats_mean_and_count() {
    let mut stats = LatencyStats::new();
    for ms in [10u64, 20, 30, 40, 50] {
        stats.record(Duration::from_millis(ms));
    }
    assert_eq!(stats.count(), 5);
    assert_eq!(stats.mean(), Some(Duration::from_millis(30)));

    let p50 = stats.p50().unwrap();
    assert!(p50 >= Duration::from_millis(10) && p50 <= Duration::from_millis(50));
    assert!(stats.p99().unwrap() >= p50);
}

#[test]
fn latency_stats_merge() {
    let mut a = LatencyStats::new();
    a.record(Duration::from_millis(5));
    let mut b = LatencyStats::new();
    b.record(Duration::from_millis(15));
    b.record(Duration::from_millis(25));

    a.merge(&b);
    assert_eq!(a.count(), 3);
    assert!(a.format_report().contains("p50"));
}

#[test]
fn robot_error_display_and_from_io() {
    let err = RobotError::Config("bad value".into());
    assert!(err.to_string().contains("bad value"));

    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let converted: RobotError = io.into();
    assert!(matches!(converted, RobotError::Io(_)));
}
