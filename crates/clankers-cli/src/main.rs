#[tokio::main]
async fn main() -> anyhow::Result<()> {
    clankers_cli::run().await
}
