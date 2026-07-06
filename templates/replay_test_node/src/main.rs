use clankers::prelude::*;

#[clankers::node]
async fn main(_ctx: RobotContext) -> RobotResult<()> {
    tracing::info!("replay test node — run `clankers test`");
    Ok(())
}

#[cfg(test)]
mod tests {
    use clankers::prelude::*;

    #[clankers::replay_test("test_data/camera_sample.mcap")]
    async fn replay_smoke(ctx: ReplayContext) -> RobotResult<()> {
        let result = ctx.run_replay(|_msg| async { Ok(()) }).await?;
        assert_no_panics(&result)?;
        assert_dropped_messages(&result, 0)?;
        Ok(())
    }
}
