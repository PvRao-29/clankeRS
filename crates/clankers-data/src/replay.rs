use std::path::Path;
use std::time::{Duration, Instant};

use clankers_core::{LatencyStats, RobotResult};

use crate::inspect::{McapLog, McapRecord};

/// Replay engine that feeds MCAP messages in timestamp order.
pub struct Replay {
    messages: Vec<McapRecord>,
    path: String,
}

#[derive(Debug, Clone, Default)]
pub struct ReplaySummary {
    pub input_messages: u64,
    pub output_messages: u64,
    pub dropped_messages: u64,
    pub deadline_misses: u64,
}

#[derive(Debug, Clone)]
pub struct ReplayResult {
    pub summary: ReplaySummary,
    pub latency: LatencyStats,
    pub path: String,
}

impl Replay {
    pub fn from_mcap(path: impl AsRef<Path>) -> RobotResult<Self> {
        let path = path.as_ref();
        let log = McapLog::open(path)?;
        let messages = log.messages()?;
        Ok(Self {
            path: path.display().to_string(),
            messages,
        })
    }

    pub fn messages(&self) -> &[McapRecord] {
        &self.messages
    }

    pub fn topics(&self) -> Vec<String> {
        let mut topics: Vec<String> = self
            .messages
            .iter()
            .map(|m| m.topic.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        topics.sort();
        topics
    }

    /// Replay messages through a callback, measuring per-message latency.
    pub async fn run<F, Fut>(&self, mut handler: F) -> RobotResult<ReplayResult>
    where
        F: FnMut(McapRecord) -> Fut,
        Fut: std::future::Future<Output = RobotResult<()>>,
    {
        let mut latency = LatencyStats::new();
        let mut dropped = 0u64;
        let mut deadline_misses = 0u64;
        let deadline = Duration::from_millis(20);

        for msg in &self.messages {
            let start = Instant::now();
            if let Err(e) = handler(msg.clone()).await {
                tracing::warn!(error = %e, topic = %msg.topic, "replay handler error");
                dropped += 1;
            }
            let elapsed = start.elapsed();
            latency.record(elapsed);
            if elapsed > deadline {
                deadline_misses += 1;
            }
        }

        let input_count = self.messages.len() as u64;
        Ok(ReplayResult {
            summary: ReplaySummary {
                input_messages: input_count,
                output_messages: input_count.saturating_sub(dropped),
                dropped_messages: dropped,
                deadline_misses,
            },
            latency,
            path: self.path.clone(),
        })
    }

    pub fn format_summary(result: &ReplayResult) -> String {
        format!(
            "Replay summary:\n  file: {}\n  input messages: {}\n  output messages: {}\n  dropped messages: {}\n  deadline misses: {}\n\n{}",
            result.path,
            result.summary.input_messages,
            result.summary.output_messages,
            result.summary.dropped_messages,
            result.summary.deadline_misses,
            result.latency.format_report(),
        )
    }
}
