use clankers_core::RobotResult;

use crate::inspect::{inspect_file, InspectReport};

#[derive(Debug, Clone)]
pub struct CompareReport {
    pub expected: InspectReport,
    pub actual: InspectReport,
    pub topic_diffs: Vec<TopicDiff>,
}

#[derive(Debug, Clone)]
pub struct TopicDiff {
    pub topic: String,
    pub expected_count: u64,
    pub actual_count: u64,
    pub count_match: bool,
}

pub fn compare_logs(expected: &str, actual: &str) -> RobotResult<CompareReport> {
    let expected_report = inspect_file(std::path::Path::new(expected))?;
    let actual_report = inspect_file(std::path::Path::new(actual))?;

    let all_topics: std::collections::BTreeSet<String> = expected_report
        .topics
        .iter()
        .map(|t| t.name.clone())
        .chain(actual_report.topics.iter().map(|t| t.name.clone()))
        .collect();

    let expected_map: std::collections::HashMap<_, _> = expected_report
        .topics
        .iter()
        .map(|t| (t.name.clone(), t.message_count))
        .collect();
    let actual_map: std::collections::HashMap<_, _> = actual_report
        .topics
        .iter()
        .map(|t| (t.name.clone(), t.message_count))
        .collect();

    let topic_diffs: Vec<TopicDiff> = all_topics
        .into_iter()
        .map(|topic| {
            let expected_count = expected_map.get(&topic).copied().unwrap_or(0);
            let actual_count = actual_map.get(&topic).copied().unwrap_or(0);
            TopicDiff {
                count_match: expected_count == actual_count,
                topic,
                expected_count,
                actual_count,
            }
        })
        .collect();

    Ok(CompareReport {
        expected: expected_report,
        actual: actual_report,
        topic_diffs,
    })
}

pub fn format_compare_report(report: &CompareReport) -> String {
    let mut out = format!(
        "Compare:\n  expected: {}\n  actual: {}\n\nTopic diffs:\n",
        report.expected.path, report.actual.path
    );
    for diff in &report.topic_diffs {
        let status = if diff.count_match { "ok" } else { "MISMATCH" };
        out.push_str(&format!(
            "  {} {} expected={} actual={}\n",
            status, diff.topic, diff.expected_count, diff.actual_count
        ));
    }
    out
}
