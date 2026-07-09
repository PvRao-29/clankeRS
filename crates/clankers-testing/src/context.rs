use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use clankers_core::{LatencyStats, RobotResult};
use clankers_data::{Replay, ReplayResult};

/// Aggregated per-frame inference accounting across a replay run.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AggregatedInferenceStats {
    pub frame_count: u32,
    pub total_copies: usize,
    pub total_allocations: usize,
    pub total_bytes_copied: usize,
    pub latency: LatencyStats,
}

impl AggregatedInferenceStats {
    pub fn record_frame(
        &mut self,
        latency: Duration,
        copies: usize,
        allocations: usize,
        bytes_copied: usize,
    ) {
        self.frame_count += 1;
        self.total_copies += copies;
        self.total_allocations += allocations;
        self.total_bytes_copied += bytes_copied;
        self.latency.record(latency);
    }
}

/// Context for replay-based tests.
pub struct ReplayContext {
    pub mcap_path: PathBuf,
    pub deterministic: bool,
    /// Per-frame inference latencies recorded during a replay, aggregated by
    /// [`latency`](ReplayContext::latency). Interior-mutable so the replay
    /// handler (which borrows the context by shared reference) can feed it.
    frame_latency: Mutex<LatencyStats>,
    /// Per-frame copy/allocation accounting from inference runs.
    frame_inference: Mutex<AggregatedInferenceStats>,
}

#[derive(Debug, Clone)]
pub struct ReplayTestResult {
    pub replay: ReplayResult,
    pub panics: u32,
    pub topics_seen: Vec<String>,
}

impl ReplayContext {
    pub fn new(mcap_path: impl Into<PathBuf>) -> Self {
        Self {
            mcap_path: mcap_path.into(),
            deterministic: true,
            frame_latency: Mutex::new(LatencyStats::new()),
            frame_inference: Mutex::new(AggregatedInferenceStats::default()),
        }
    }

    /// Record one frame's inference latency (e.g. `InferenceStats::latency` from
    /// `clankers_ml`) so [`latency`](Self::latency) can report an aggregate over
    /// the whole replay.
    pub fn record_frame_latency(&self, latency: Duration) {
        self.frame_latency.lock().unwrap().record(latency);
    }

    /// Record per-frame inference accounting (e.g. fields from
    /// `clankers_ml::InferenceStats`) alongside latency.
    pub fn record_frame_inference(
        &self,
        latency: Duration,
        copies: usize,
        allocations: usize,
        bytes_copied: usize,
    ) {
        self.frame_inference.lock().unwrap().record_frame(
            latency,
            copies,
            allocations,
            bytes_copied,
        );
    }

    pub async fn run_replay<F, Fut>(&self, mut handler: F) -> RobotResult<ReplayTestResult>
    where
        F: FnMut(clankers_data::McapRecord) -> Fut,
        Fut: std::future::Future<Output = RobotResult<()>>,
    {
        let replay = Replay::from_mcap(&self.mcap_path)?;
        let mut topics_seen = std::collections::HashSet::new();

        let result = replay
            .run(|msg| {
                topics_seen.insert(msg.topic.clone());
                handler(msg)
            })
            .await?;

        let mut topics: Vec<_> = topics_seen.into_iter().collect();
        topics.sort();

        Ok(ReplayTestResult {
            replay: result,
            panics: 0,
            topics_seen: topics,
        })
    }

    /// The aggregate of every latency passed to
    /// [`record_frame_latency`](Self::record_frame_latency) during the replay.
    ///
    /// For end-to-end replay timing (per message, measured by the replay engine),
    /// use `result.replay.latency` from [`run_replay`](Self::run_replay) instead.
    pub fn latency(&self) -> LatencyStats {
        self.frame_latency.lock().unwrap().clone()
    }

    /// Aggregated copy/allocation accounting from
    /// [`record_frame_inference`](Self::record_frame_inference).
    pub fn inference_stats(&self) -> AggregatedInferenceStats {
        self.frame_inference.lock().unwrap().clone()
    }
}
