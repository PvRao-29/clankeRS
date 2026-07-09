//! # Replay-based testing
//!
//! Run node logic against recorded MCAP fixtures and assert on topics, latency,
//! and dropped messages.
//!
//! Use the [`clankers::replay_test`](https://docs.rs/clankers/latest/clankers/attr.replay_test.html)
//! attribute macro from the facade, or call [`ReplayContext`] directly in `#[tokio::test]`:
//!
//! ```
//! use std::time::Duration;
//! use clankers_testing::{
//!     assert_max_latency, assert_no_panics, assert_topic_exists, ReplayContext,
//! };
//!
//! #[tokio::test]
//! async fn camera_log_replays_cleanly() {
//!     let ctx = ReplayContext::new("tests/fixtures/camera_log.mcap");
//!     let result = ctx.run_replay(|_msg| async { Ok(()) }).await.unwrap();
//!
//!     assert_no_panics(&result).unwrap();
//!     assert_topic_exists(&result, "/camera/image_raw").unwrap();
//!     assert_max_latency(&result, Duration::from_secs(10)).unwrap();
//! }
//! ```
//!
//! ## Assertion helpers
//!
//! | Function | Checks |
//! |----------|--------|
//! | [`assert_topic_exists`] | A topic appeared during replay |
//! | [`assert_no_panics`] | Handler completed without panics |
//! | [`assert_dropped_messages`] | Drop count within budget |
//! | [`assert_max_latency`] | p99 latency under a ceiling |

pub mod assertions;
pub mod context;

pub use assertions::{
    assert_dropped_messages, assert_max_latency, assert_no_panics, assert_topic_exists,
};
pub use context::{AggregatedInferenceStats, ReplayContext, ReplayTestResult};
