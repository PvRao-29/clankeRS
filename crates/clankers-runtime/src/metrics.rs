use clankers_core::LatencyStats;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct RuntimeMetrics {
    pub callback_latency: LatencyStats,
    pub inference_latency: LatencyStats,
    pub publish_latency: LatencyStats,
    pub dropped_messages: AtomicU64,
    pub deadline_misses: AtomicU64,
    pub panic_count: AtomicU64,
    pub max_queue_depth: usize,
    pub current_queue_depth: AtomicU64,
}

impl RuntimeMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_dropped(&self, n: u64) {
        self.dropped_messages.fetch_add(n, Ordering::Relaxed);
    }

    pub fn record_deadline_miss(&self) {
        self.deadline_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn format_report(&self) -> String {
        format!(
            "Runtime metrics:\n  dropped messages: {}\n  deadline misses: {}\n  panic count: {}\n  queue depth: {}\n{}",
            self.dropped_messages.load(Ordering::Relaxed),
            self.deadline_misses.load(Ordering::Relaxed),
            self.panic_count.load(Ordering::Relaxed),
            self.current_queue_depth.load(Ordering::Relaxed),
            self.callback_latency.format_report(),
        )
    }
}
