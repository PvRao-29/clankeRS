//! # clankeRS core primitives
//!
//! Shared types used by every other clankeRS crate: configuration, errors, timestamps,
//! and the [`RobotContext`] that nodes receive at startup.
//!
//! Most applications import these through the [`clankers`](https://docs.rs/clankers) facade
//! rather than depending on `clankers-core` directly.
//!
//! ## Configuration
//!
//! Project settings live in `clankeRS.toml`. Load them with [`RobotContext::from_work_dir`]:
//!
//! ```no_run
//! use clankers_core::RobotContext;
//!
//! let ctx = RobotContext::from_work_dir(".")?;
//! let model_cfg = ctx.model_config("detector")?;
//! let model_path = ctx.resolve_path(&model_cfg.path);
//! # Ok::<(), clankers_core::RobotError>(())
//! ```
//!
//! ## Key types
//!
//! | Type | Role |
//! |------|------|
//! | [`RobotContext`] | Node name, config, and path resolution |
//! | [`ClankeRSConfig`] / [`ModelConfig`] | Parsed `clankeRS.toml` sections |
//! | [`RobotError`] / [`RobotResult`] | Unified error type |
//! | [`Timestamp`] / [`TopicName`] | Time and ROS-style topic identifiers |
//! | [`LatencyStats`] | Rolling latency percentiles |

pub mod config;
pub mod context;
pub mod error;
pub mod latency;
pub mod types;

pub use config::{ClankeRSConfig, ModelBackendKind, ModelConfig};
pub use context::RobotContext;
pub use error::{RobotError, RobotResult};
pub use latency::LatencyStats;
pub use types::{FrameId, NodeName, Rate, Timestamp, TopicName};
