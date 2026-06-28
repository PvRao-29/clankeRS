use std::time::Instant;

use clankers::prelude::*;

#[clankers::node]
async fn main(ctx: RobotContext) -> Result<()> {
    let node = RobotNode::new(ctx.node_name().as_str()).await?;
    let mut camera = node
        .subscribe::<ImageMsg>("/camera/image_raw", QosProfile::sensor_data())
        .await?;
    let detections_pub = node
        .publish::<DetectionArray>("/detections", QosProfile::default())
        .await?;

    let model_cfg = ctx.model_config("detector")?;
    let model_path = ctx.resolve_path(&model_cfg.path);
    let model = Model::load(&model_path)?;

    tracing::info!("clankeRS node: {}", ctx.node_name());
    tracing::info!("Loaded model {}", model_path.display());

    while let Some(frame) = camera.next().await {
        let start = Instant::now();
        let tensor = ImageTensor::from_ros_msg(&frame)?
            .resize(224, 224)?
            .normalize_imagenet()?
            .to_nchw()?;

        let output = model.run(&tensor.to_vec())?;
        let detections = output_to_detections(&output);

        detections_pub
            .publish(DetectionArray {
                stamp_nanos: Timestamp::now().as_nanos(),
                frame_id: frame.frame_id.clone(),
                detections,
            })
            .await?;

        tracing::debug!(latency_ms = start.elapsed().as_secs_f64() * 1000.0, "inference");
    }

    Ok(())
}

fn output_to_detections(output: &[f32]) -> Vec<Detection> {
    output
        .iter()
        .enumerate()
        .filter(|(_, &score)| score > 0.1)
        .map(|(class_id, &score)| Detection {
            class_id: class_id as u32,
            class_name: format!("class_{class_id}"),
            score,
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        })
        .collect()
}
