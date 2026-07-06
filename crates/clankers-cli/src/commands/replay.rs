use anyhow::Result;
use clankers_data::{Replay, ReplayResult};

pub async fn execute(file: &str, node: Option<&str>) -> Result<()> {
    if let Some(node_path) = node {
        println!("Replaying through node: {node_path}");
    }

    let replay = Replay::from_mcap(file).map_err(|e| anyhow::anyhow!("{e}"))?;
    let result = replay
        .run(|_msg| async { Ok(()) })
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    print_summary(&result);
    Ok(())
}

fn print_summary(result: &ReplayResult) {
    println!("{}", clankers_data::Replay::format_summary(result));
}
