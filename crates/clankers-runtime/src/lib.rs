//! # Execution, scheduling, and observability
//!
//! Runtime helpers for clankeRS nodes: tracing initialization, deadlines, and
//! [`RuntimeMetrics`] collection.
//!
//! Most nodes call [`runtime::init_tracing`] via the [`clankers::node`](https://docs.rs/clankers/latest/clankers/attr.node.html)
//! macro. Use [`RobotRuntime`] when you need explicit scheduling or metrics hooks.
//!
//! ```no_run
//! use clankers_runtime::RobotRuntime;
//!
//! let runtime = RobotRuntime::builder().build()?;
//! println!("{}", runtime.metrics().format_report());
//! # Ok::<(), clankers_core::RobotError>(())
//! ```

pub mod metrics;
pub mod runtime;

pub use metrics::RuntimeMetrics;
pub use runtime::RobotRuntime;
