//! Core primitives shared across the clankeRS SDK.

pub mod config;
pub mod context;
pub mod error;
pub mod latency;
pub mod types;

pub use config::ClankeRSConfig;
pub use context::RobotContext;
pub use error::{RobotError, RobotResult};
pub use latency::LatencyStats;
pub use types::{FrameId, NodeName, Rate, Timestamp, TopicName};
