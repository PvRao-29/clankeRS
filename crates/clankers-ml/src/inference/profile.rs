//! Per-run inference accounting.
//!
//! [`InferenceStats`] is the engine's answer to "what did that inference cost?".
//! The headline metric is [`InferenceStats::copies`] — the number of copies the
//! *engine* made to adapt the caller's inputs to what the backend expects. On the
//! zero-copy path this is `0`, which is the property the engine's tests assert.
//! Copies the backend performs internally (IO binding, producing outputs) are
//! reported separately in [`InferenceStats::backend`] so the two are never
//! conflated.

use std::time::Duration;

use crate::backend::BackendRunStats;

/// What a single inference run cost.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InferenceStats {
    /// Wall-clock time spent inside the backend's `run` (excludes input binding).
    pub latency: Duration,
    /// Copies clankeRS made adapting inputs before handing them to the backend.
    /// `0` means every input was bound by borrow with no clankeRS-side copy.
    pub clankers_copies: usize,
    /// Bytes moved by those clankeRS-side input conversions.
    pub clankers_bytes_copied: usize,
    /// Arena allocations performed this run (input conversions + output buffers).
    pub allocations: usize,
    /// Total bytes those arena allocations reserved.
    pub bytes_allocated: usize,
    /// Copies the backend reported performing internally.
    pub backend: BackendRunStats,
}

impl InferenceStats {
    /// Latency in whole microseconds.
    pub fn latency_us(&self) -> u128 {
        self.latency.as_micros()
    }

    /// Latency in fractional milliseconds.
    pub fn latency_ms(&self) -> f64 {
        self.latency.as_secs_f64() * 1000.0
    }

    /// Whether clankeRS adapted no inputs before handing them to the backend.
    pub fn is_zero_copy(&self) -> bool {
        self.clankers_copies == 0
    }

    /// clankeRS conversion copies plus copies the backend reported internally.
    pub fn total_copies(&self) -> usize {
        self.clankers_copies + self.backend.backend_copies
    }

    /// Record one clankeRS-side input conversion copy of `bytes` bytes.
    pub(crate) fn record_conversion_copy(&mut self, bytes: usize) {
        self.clankers_copies += 1;
        self.clankers_bytes_copied += bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_copy_by_default() {
        let stats = InferenceStats::default();
        assert!(stats.is_zero_copy());
        assert_eq!(stats.total_copies(), 0);
    }

    #[test]
    fn conversion_copies_accumulate() {
        let mut stats = InferenceStats::default();
        stats.record_conversion_copy(64);
        stats.record_conversion_copy(16);
        assert_eq!(stats.clankers_copies, 2);
        assert_eq!(stats.clankers_bytes_copied, 80);
        assert!(!stats.is_zero_copy());
    }

    #[test]
    fn total_copies_folds_in_backend_copies() {
        let mut stats = InferenceStats::default();
        stats.record_conversion_copy(8);
        stats.backend.record_copy(32);
        assert_eq!(stats.clankers_copies, 1);
        assert_eq!(stats.total_copies(), 2);
    }
}
