use anyhow::Result;
use clankers_data::{format_inspect_report, McapLog};

pub fn execute(file: &str) -> Result<()> {
    let log = McapLog::open(file).map_err(|e| anyhow::anyhow!("{e}"))?;
    print!("{}", format_inspect_report(log.report()));
    Ok(())
}
