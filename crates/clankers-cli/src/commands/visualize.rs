use anyhow::Result;
use clankers_data::{format_inspect_report, McapLog};

pub fn execute(file: &str) -> Result<()> {
    let log = McapLog::open(file).map_err(|e| anyhow::anyhow!("{e}"))?;
    let report = log.report();

    println!("clankeRS visualize — Foxglove/Rerun hook\n");
    print!("{}", format_inspect_report(report));
    println!(
        "\nVisualization:\n  Open {} in Foxglove Studio (https://foxglove.dev)\n  Or convert topics for Rerun using your robot's message schemas.",
        file
    );
    Ok(())
}
