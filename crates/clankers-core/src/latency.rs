use std::time::Duration;

/// Latency percentile statistics.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LatencyStats {
    pub samples: Vec<Duration>,
}

impl LatencyStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, duration: Duration) {
        self.samples.push(duration);
    }

    pub fn merge(&mut self, other: &LatencyStats) {
        self.samples.extend_from_slice(&other.samples);
    }

    pub fn count(&self) -> usize {
        self.samples.len()
    }

    pub fn percentile(&self, p: f64) -> Option<Duration> {
        if self.samples.is_empty() {
            return None;
        }
        let mut sorted = self.samples.clone();
        sorted.sort();
        let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
        Some(sorted[idx.min(sorted.len() - 1)])
    }

    pub fn p50(&self) -> Option<Duration> {
        self.percentile(50.0)
    }

    pub fn p95(&self) -> Option<Duration> {
        self.percentile(95.0)
    }

    pub fn p99(&self) -> Option<Duration> {
        self.percentile(99.0)
    }

    pub fn mean(&self) -> Option<Duration> {
        if self.samples.is_empty() {
            return None;
        }
        let total: Duration = self.samples.iter().sum();
        Some(total / self.samples.len() as u32)
    }

    pub fn format_report(&self) -> String {
        if self.samples.is_empty() {
            return "Latency: (no samples)".to_string();
        }
        format!(
            "Latency:\n  p50: {}\n  p95: {}\n  p99: {}",
            format_duration(self.p50().unwrap()),
            format_duration(self.p95().unwrap()),
            format_duration(self.p99().unwrap()),
        )
    }
}

pub fn format_duration(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 1.0 {
        format!("{:.2} µs", d.as_secs_f64() * 1_000_000.0)
    } else if ms < 1000.0 {
        format!("{ms:.1} ms")
    } else {
        format!("{:.2} s", d.as_secs_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_computation() {
        let mut stats = LatencyStats::new();
        for ms in [1, 2, 3, 4, 100] {
            stats.record(Duration::from_millis(ms));
        }
        assert_eq!(stats.p50(), Some(Duration::from_millis(3)));
        assert_eq!(stats.p95(), Some(Duration::from_millis(100)));
    }
}
