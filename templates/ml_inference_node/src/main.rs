use clankers::prelude::*;

#[clankers::node]
async fn main(ctx: RobotContext) -> RobotResult<()> {
    let model_cfg = ctx.model_config("policy")?;
    let model = Model::load(ctx.resolve_path(&model_cfg.path))?;
    tracing::info!(backend = %model.metadata().backend, "loaded policy model");
    Ok(())
}
