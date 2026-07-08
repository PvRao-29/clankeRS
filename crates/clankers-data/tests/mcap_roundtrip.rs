//! Integration tests: write an MCAP, read it back, inspect it, replay it,
//! and compare two logs.

use std::sync::atomic::{AtomicU64, Ordering};

use clankers_core::Timestamp;
use clankers_data::{compare_logs, McapLog, McapWriter, Replay};

/// Write a small MCAP with two topics (3 msgs on /a, 2 on /b) and return its path.
fn write_sample(path: &std::path::Path) {
    let mut w = McapWriter::create(path).unwrap();
    for i in 0..3u64 {
        w.write_message(
            "/a",
            "sensor_msgs/Image",
            "json",
            format!("a{i}").as_bytes(),
            Timestamp::from_nanos(i * 1_000_000),
        )
        .unwrap();
    }
    for i in 0..2u64 {
        w.write_message(
            "/b",
            "std_msgs/String",
            "json",
            format!("b{i}").as_bytes(),
            Timestamp::from_nanos(100 + i * 1_000_000),
        )
        .unwrap();
    }
    w.finish().unwrap();
}

#[test]
fn write_inspect_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("log.mcap");
    write_sample(&path);

    let log = McapLog::open(&path).unwrap();
    let report = log.report();

    let total: u64 = report.topics.iter().map(|t| t.message_count).sum();
    assert_eq!(total, 5);

    let topic_a = report.topics.iter().find(|t| t.name == "/a").unwrap();
    assert_eq!(topic_a.message_count, 3);
    // message_type is the MCAP message encoding; the schema name is separate.
    assert_eq!(topic_a.message_type, "json");
    assert_eq!(topic_a.schema.as_deref(), Some("sensor_msgs/Image"));

    assert!(report.start_time.is_some());
    assert!(report.end_time.is_some());
    assert!(report.end_time.unwrap() >= report.start_time.unwrap());

    // All messages, and per-topic filtering.
    assert_eq!(log.messages().unwrap().len(), 5);
    assert_eq!(log.topic_messages("/a").unwrap().len(), 3);
    assert_eq!(log.topic_messages("/b").unwrap().len(), 2);
}

#[test]
fn messages_are_sorted_by_log_time() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("log.mcap");
    write_sample(&path);

    let msgs = McapLog::open(&path).unwrap().messages().unwrap();
    for pair in msgs.windows(2) {
        assert!(pair[0].log_time <= pair[1].log_time);
    }
}

#[tokio::test]
async fn replay_runs_all_messages() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("log.mcap");
    write_sample(&path);

    let replay = Replay::from_mcap(&path).unwrap();
    assert_eq!(replay.topics(), vec!["/a".to_string(), "/b".to_string()]);

    let seen = AtomicU64::new(0);
    let result = replay
        .run(|_msg| {
            seen.fetch_add(1, Ordering::Relaxed);
            async { Ok(()) }
        })
        .await
        .unwrap();

    assert_eq!(seen.load(Ordering::Relaxed), 5);
    assert_eq!(result.summary.input_messages, 5);
    assert_eq!(result.summary.output_messages, 5);
    assert_eq!(result.summary.dropped_messages, 0);
    assert_eq!(result.latency.count(), 5);
}

#[tokio::test]
async fn replay_counts_handler_errors_as_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("log.mcap");
    write_sample(&path);

    let replay = Replay::from_mcap(&path).unwrap();
    let result = replay
        .run(|msg| async move {
            if msg.topic == "/b" {
                Err(clankers_core::RobotError::Other("boom".into()))
            } else {
                Ok(())
            }
        })
        .await
        .unwrap();

    // 2 messages on /b fail -> dropped 2, output = 5 - 2.
    assert_eq!(result.summary.dropped_messages, 2);
    assert_eq!(result.summary.output_messages, 3);
}

#[test]
fn compare_identical_logs_matches() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("log.mcap");
    write_sample(&path);
    let p = path.to_str().unwrap();

    let report = compare_logs(p, p).unwrap();
    assert!(!report.topic_diffs.is_empty());
    assert!(report.topic_diffs.iter().all(|d| d.count_match));
}
