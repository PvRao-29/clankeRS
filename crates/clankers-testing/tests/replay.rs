//! Integration tests for the replay-based testing framework and assertions.

use std::time::Duration;

use clankers_data::sample::generate_camera_log;
use clankers_testing::{
    assert_dropped_messages, assert_max_latency, assert_no_panics, assert_topic_exists,
    ReplayContext,
};

async fn run_sample() -> (tempfile::TempDir, clankers_testing::ReplayTestResult) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("camera_log.mcap");
    generate_camera_log(&path).unwrap();

    let ctx = ReplayContext::new(&path);
    let result = ctx.run_replay(|_msg| async { Ok(()) }).await.unwrap();
    (dir, result)
}

#[tokio::test]
async fn replay_context_sees_topics() {
    let (_dir, result) = run_sample().await;
    assert!(result
        .topics_seen
        .contains(&"/camera/image_raw".to_string()));
    assert_eq!(result.panics, 0);
    assert!(result.replay.summary.input_messages > 0);
}

#[tokio::test]
async fn replay_context_aggregates_inference_stats() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("camera_log.mcap");
    generate_camera_log(&path).unwrap();

    let ctx = ReplayContext::new(&path);
    ctx.run_replay(|_msg| {
        ctx.record_frame_inference(Duration::from_micros(500), 0, 1, 0);
        async { Ok(()) }
    })
    .await
    .unwrap();

    let stats = ctx.inference_stats();
    assert!(stats.frame_count > 0);
    assert_eq!(stats.total_allocations, stats.frame_count as usize);
}

#[tokio::test]
async fn replay_context_aggregates_frame_latency() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("camera_log.mcap");
    generate_camera_log(&path).unwrap();

    let ctx = ReplayContext::new(&path);
    // The handler feeds each frame's inference latency into the context, which
    // aggregates them (as a perception replay test using InferenceStats would).
    ctx.run_replay(|_msg| {
        ctx.record_frame_latency(Duration::from_micros(500));
        async { Ok(()) }
    })
    .await
    .unwrap();

    let latency = ctx.latency();
    assert!(
        latency.count() > 0,
        "per-frame latencies should be recorded"
    );
}

#[tokio::test]
async fn assertions_pass_on_clean_replay() {
    let (_dir, result) = run_sample().await;
    assert_topic_exists(&result, "/camera/image_raw").unwrap();
    assert_no_panics(&result).unwrap();
    assert_dropped_messages(&result, 0).unwrap();
    assert_max_latency(&result, Duration::from_secs(10)).unwrap();
}

#[tokio::test]
async fn assert_topic_exists_fails_for_missing_topic() {
    let (_dir, result) = run_sample().await;
    assert!(assert_topic_exists(&result, "/does_not_exist").is_err());
}

#[tokio::test]
async fn assert_dropped_messages_fails_when_exceeded() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("camera_log.mcap");
    generate_camera_log(&path).unwrap();

    // Every message errors -> all dropped.
    let result = ReplayContext::new(&path)
        .run_replay(|_msg| async { Err(clankers_core::RobotError::Other("fail".into())) })
        .await
        .unwrap();

    assert!(result.replay.summary.dropped_messages > 0);
    assert!(assert_dropped_messages(&result, 0).is_err());
}
