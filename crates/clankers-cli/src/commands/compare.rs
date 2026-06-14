use anyhow::Result;
use clankers_data::{compare_logs, format_compare_report};

pub fn execute(expected: &str, actual: &str) -> Result<()> {
    let report = compare_logs(expected, actual).map_err(|e| anyhow::anyhow!("{e}"))?;
    print!("{}", format_compare_report(&report));
    Ok(())
}
