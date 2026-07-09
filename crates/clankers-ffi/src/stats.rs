//! [`ClankersInferenceStats`] conversions.

use clankers_ml::inference::InferenceStats;

use crate::ClankersInferenceStats;

pub fn stats_to_c(stats: &InferenceStats) -> ClankersInferenceStats {
    ClankersInferenceStats {
        latency_us: stats.latency.as_micros() as u64,
        copies: stats.clankers_copies,
        bytes_copied: stats.clankers_bytes_copied,
        allocations: stats.allocations,
        bytes_allocated: stats.bytes_allocated,
        backend_latency_us: 0,
        backend_copies: stats.backend.backend_copies,
        backend_bytes_copied: stats.backend.backend_bytes_copied,
    }
}
