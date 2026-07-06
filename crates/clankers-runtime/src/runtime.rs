use std::sync::Arc;
use std::time::Duration;

use clankers_core::RobotResult;

use crate::metrics::RuntimeMetrics;

pub struct RobotRuntime {
    pub max_queue_depth: usize,
    pub deadline: Option<Duration>,
    pub metrics: Arc<RuntimeMetrics>,
    pub latency_tracing: bool,
}

pub struct RobotRuntimeBuilder {
    max_queue_depth: usize,
    deadline: Option<Duration>,
    latency_tracing: bool,
}

impl RobotRuntime {
    pub fn builder() -> RobotRuntimeBuilder {
        RobotRuntimeBuilder {
            max_queue_depth: 4,
            deadline: None,
            latency_tracing: false,
        }
    }

    pub fn metrics(&self) -> &RuntimeMetrics {
        &self.metrics
    }
}

impl RobotRuntimeBuilder {
    pub fn max_queue_depth(mut self, depth: usize) -> Self {
        self.max_queue_depth = depth;
        self
    }

    pub fn deadline(mut self, d: Duration) -> Self {
        self.deadline = Some(d);
        self
    }

    pub fn enable_latency_tracing(mut self, enable: bool) -> Self {
        self.latency_tracing = enable;
        self
    }

    pub fn build(self) -> RobotResult<RobotRuntime> {
        Ok(RobotRuntime {
            max_queue_depth: self.max_queue_depth,
            deadline: self.deadline,
            metrics: Arc::new(RuntimeMetrics::new()),
            latency_tracing: self.latency_tracing,
        })
    }
}

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}
