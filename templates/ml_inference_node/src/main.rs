use clankers::prelude::*;

#[clankers::node]
async fn main(ctx: RobotContext) -> RobotResult<()> {
    let model_cfg = ctx.model_config("policy")?;
    let model = ModelBuilder::from_config(&model_cfg, ctx.resolve_path(&model_cfg.path))?.build()?;
    tracing::info!(backend = %model.metadata().backend, "loaded policy model");
    Ok(())
}
