use std::time::Duration;

use clankers_core::RobotResult;

use crate::context::ReplayTestResult;

pub fn assert_topic_exists(result: &ReplayTestResult, topic: &str) -> RobotResult<()> {
    if result.topics_seen.iter().any(|t| t == topic) {
        Ok(())
    } else {
        Err(clankers_core::RobotError::Other(format!(
            "expected topic '{topic}' not seen during replay; got: {:?}",
            result.topics_seen
        )))
    }
}

pub fn assert_max_latency(result: &ReplayTestResult, max: Duration) -> RobotResult<()> {
    if let Some(p99) = result.replay.latency.p99() {
        if p99 > max {
            return Err(clankers_core::RobotError::Other(format!(
                "p99 latency {:?} exceeds max {:?}",
                p99, max
            )));
        }
    }
    Ok(())
}

pub fn assert_no_panics(result: &ReplayTestResult) -> RobotResult<()> {
    if result.panics > 0 {
        return Err(clankers_core::RobotError::Other(format!(
            "expected no panics, got {}",
            result.panics
        )));
    }
    Ok(())
}

pub fn assert_dropped_messages(result: &ReplayTestResult, max: u64) -> RobotResult<()> {
    if result.replay.summary.dropped_messages > max {
        return Err(clankers_core::RobotError::Other(format!(
            "dropped {} messages, max allowed {}",
            result.replay.summary.dropped_messages, max
        )));
    }
    Ok(())
}
