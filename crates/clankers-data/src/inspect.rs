use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use mcap::MessageStream;

use clankers_core::{RobotError, RobotResult, Timestamp};

/// Summary of an MCAP file's contents.
#[derive(Debug, Clone, Default)]
pub struct InspectReport {
    pub path: String,
    pub topics: Vec<TopicInfo>,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
}

#[derive(Debug, Clone)]
pub struct TopicInfo {
    pub name: String,
    pub message_type: String,
    pub message_count: u64,
    pub schema: Option<String>,
}

/// Read-only handle to an MCAP log file.
pub struct McapLog {
    path: std::path::PathBuf,
    data: Vec<u8>,
    report: InspectReport,
}

impl McapLog {
    pub fn open(path: impl AsRef<Path>) -> RobotResult<Self> {
        let path = path.as_ref().to_path_buf();
        let data = fs::read(&path)
            .map_err(|e| RobotError::Data(format!("read {}: {e}", path.display())))?;
        let report = inspect_bytes(&path, &data)?;
        Ok(Self { path, data, report })
    }

    pub fn report(&self) -> &InspectReport {
        &self.report
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn messages(&self) -> RobotResult<Vec<McapRecord>> {
        read_all_messages(&self.data)
    }

    pub fn topic_messages(&self, topic: &str) -> RobotResult<Vec<McapRecord>> {
        Ok(self
            .messages()?
            .into_iter()
            .filter(|m| m.topic == topic)
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct McapRecord {
    pub topic: String,
    pub log_time: Timestamp,
    pub publish_time: Timestamp,
    pub data: Vec<u8>,
    pub schema_name: Option<String>,
}

pub fn inspect_file(path: &Path) -> RobotResult<InspectReport> {
    let data =
        fs::read(path).map_err(|e| RobotError::Data(format!("read {}: {e}", path.display())))?;
    inspect_bytes(path, &data)
}

fn inspect_bytes(path: &Path, data: &[u8]) -> RobotResult<InspectReport> {
    if let Ok(Some(summary)) = mcap::Summary::read(data) {
        let mut topics = Vec::new();
        let counts = summary
            .stats
            .as_ref()
            .map(|s| s.channel_message_counts.clone())
            .unwrap_or_default();

        for (channel_id, channel) in &summary.channels {
            let message_count = counts.get(channel_id).copied().unwrap_or(0);
            let schema = channel
                .schema
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| channel.message_encoding.clone());
            topics.push(TopicInfo {
                name: channel.topic.clone(),
                message_type: channel.message_encoding.clone(),
                message_count,
                schema: Some(schema),
            });
        }
        topics.sort_by(|a, b| a.name.cmp(&b.name));

        let (start_time, end_time) = summary
            .stats
            .as_ref()
            .map(|s| {
                (
                    Some(Timestamp::from_nanos(s.message_start_time)),
                    Some(Timestamp::from_nanos(s.message_end_time)),
                )
            })
            .unwrap_or((None, None));

        if !topics.is_empty() {
            return Ok(InspectReport {
                path: path.display().to_string(),
                topics,
                start_time,
                end_time,
            });
        }
    }

    // Fallback: scan all messages
    let mut topic_counts: HashMap<String, u64> = HashMap::new();
    let mut topic_types: HashMap<String, String> = HashMap::new();
    let mut topic_schemas: HashMap<String, String> = HashMap::new();
    let mut start: Option<u64> = None;
    let mut end: Option<u64> = None;

    for msg in MessageStream::new(data).map_err(|e| RobotError::Data(e.to_string()))? {
        let msg = msg.map_err(|e| RobotError::Data(e.to_string()))?;
        let topic = msg.channel.topic.clone();
        *topic_counts.entry(topic.clone()).or_insert(0) += 1;
        topic_types
            .entry(topic.clone())
            .or_insert_with(|| msg.channel.message_encoding.clone());
        if let Some(schema) = &msg.channel.schema {
            topic_schemas
                .entry(topic)
                .or_insert_with(|| schema.name.clone());
        }
        start = Some(start.map_or(msg.log_time, |s| s.min(msg.log_time)));
        end = Some(end.map_or(msg.log_time, |e| e.max(msg.log_time)));
    }

    let topics: Vec<TopicInfo> = topic_counts
        .into_iter()
        .map(|(name, message_count)| TopicInfo {
            message_type: topic_types.get(&name).cloned().unwrap_or_default(),
            schema: topic_schemas.get(&name).cloned(),
            name,
            message_count,
        })
        .collect();

    Ok(InspectReport {
        path: path.display().to_string(),
        topics,
        start_time: start.map(Timestamp::from_nanos),
        end_time: end.map(Timestamp::from_nanos),
    })
}

pub fn read_all_messages(data: &[u8]) -> RobotResult<Vec<McapRecord>> {
    let mut messages = Vec::new();
    for msg in MessageStream::new(data).map_err(|e| RobotError::Data(e.to_string()))? {
        let msg = msg.map_err(|e| RobotError::Data(e.to_string()))?;
        messages.push(McapRecord {
            topic: msg.channel.topic.clone(),
            log_time: Timestamp::from_nanos(msg.log_time),
            publish_time: Timestamp::from_nanos(msg.publish_time),
            data: msg.data.to_vec(),
            schema_name: msg.channel.schema.as_ref().map(|s| s.name.clone()),
        });
    }
    messages.sort_by_key(|m| m.log_time.as_nanos());
    Ok(messages)
}

pub fn format_datetime(ts: &Timestamp) -> String {
    ts.to_datetime()
        .map(|dt: DateTime<Utc>| dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        .unwrap_or_else(|| format!("{}ns", ts.as_nanos()))
}

pub fn format_inspect_report(report: &InspectReport) -> String {
    let mut out = format!("File: {}\n\nTopics:\n", report.path);
    for t in &report.topics {
        let schema = t.schema.as_deref().unwrap_or(t.message_type.as_str());
        out.push_str(&format!(
            "  {:<24} {:<24} {:>8} messages\n",
            t.name, schema, t.message_count
        ));
    }
    out.push_str("\nTime range:\n");
    match (&report.start_time, &report.end_time) {
        (Some(s), Some(e)) => {
            out.push_str(&format!("  start: {}\n", format_datetime(s)));
            out.push_str(&format!("  end:   {}\n", format_datetime(e)));
        }
        _ => out.push_str("  (no messages)\n"),
    }
    out
}
