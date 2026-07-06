use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use crate::error::{RobotError, RobotResult};

/// Nanosecond-precision robot timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn now() -> Self {
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        )
    }

    pub fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    pub fn as_nanos(&self) -> u64 {
        self.0
    }

    pub fn as_duration(&self) -> Duration {
        Duration::from_nanos(self.0)
    }

    pub fn to_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(
            (self.0 / 1_000_000_000) as i64,
            (self.0 % 1_000_000_000) as u32,
        )
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_datetime() {
            Some(dt) => write!(f, "{dt}"),
            None => write!(f, "{}ns", self.0),
        }
    }
}

/// Strongly typed ROS topic name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TopicName(String);

impl TopicName {
    pub fn new(name: impl Into<String>) -> RobotResult<Self> {
        let name = name.into();
        if !name.starts_with('/') {
            return Err(RobotError::Topic(format!(
                "topic name must start with '/', got '{name}'"
            )));
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TopicName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TopicName {
    type Err = RobotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Strongly typed node name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeName(String);

impl NodeName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Coordinate frame identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FrameId(String);

impl FrameId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FrameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Target publish/subscribe rate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rate {
    pub hz: f64,
}

impl Rate {
    pub fn new(hz: f64) -> Self {
        Self { hz }
    }

    pub fn period(&self) -> Duration {
        if self.hz <= 0.0 {
            Duration::MAX
        } else {
            Duration::from_secs_f64(1.0 / self.hz)
        }
    }
}

/// Execution deadline for a callback or inference step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deadline(pub Duration);

impl Deadline {
    pub fn from_millis(ms: u64) -> Self {
        Self(Duration::from_millis(ms))
    }
}
