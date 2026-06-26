use std::path::PathBuf;

use clankers_core::{LatencyStats, RobotResult};
use clankers_data::{Replay, ReplayResult};

/// Context for replay-based tests.
pub struct ReplayContext {
    pub mcap_path: PathBuf,
    pub deterministic: bool,
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
        }
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

    pub fn latency(&self) -> &LatencyStats {
        // convenience for assertions after run
        unimplemented!("call result.replay.latency after run_replay")
    }
}
