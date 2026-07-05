use anyhow::Result;

pub fn execute(output: &str) -> Result<()> {
    println!("clankeRS record → {output}");
    println!("Hint: enable [logging] record_mcap = true in clankeRS.toml and run clankers run");
    Ok(())
}
