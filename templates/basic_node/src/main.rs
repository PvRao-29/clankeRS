use clankers::prelude::*;

#[clankers::node]
async fn main(ctx: RobotContext) -> Result<()> {
    let node = RobotNode::new(ctx.node_name().as_str()).await?;
    let mut sub = node
        .subscribe::<ImageMsg>("/chatter", QosProfile::default())
        .await?;
    let pub_ = node
        .publish::<DetectionArray>("/chatter_out", QosProfile::default())
        .await?;

    tracing::info!("clankeRS node started");
    tracing::info!("Subscribed to /chatter");
    tracing::info!("Publishing to /chatter_out");

    let mut count = 0u32;
    loop {
        if let Some(_msg) = sub.next().await {
            let detections = DetectionArray {
                stamp_nanos: Timestamp::now().as_nanos(),
                frame_id: "base".to_string(),
                detections: vec![Detection {
                    class_id: 0,
                    class_name: "hello".to_string(),
                    score: 1.0,
                    x: 0.0,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                }],
            };
            pub_.publish(detections).await?;
            count += 1;
            tracing::info!(count, "published detection");
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if count >= 10 {
            break;
        }
    }

    Ok(())
}
