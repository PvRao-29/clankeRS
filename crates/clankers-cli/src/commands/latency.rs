use anyhow::Result;
use clankers_data::Replay;

pub async fn execute(file: &str) -> Result<()> {
    let replay = Replay::from_mcap(file).map_err(|e| anyhow::anyhow!("{e}"))?;
    let result = replay
        .run(|_msg| async { Ok(()) })
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("{}", result.latency.format_report());
    Ok(())
}
