use clankers::prelude::*;

#[tokio::main]
async fn main() -> RobotResult<()> {
    clankers::runtime::init_tracing();

    let node = RobotNode::new("pubsub_minimal").await?;
    let pub_ = node
        .publish::<ImageMsg>("/camera/image_raw", QosProfile::sensor_data())
        .await?;

    tracing::info!("Publishing test images to /camera/image_raw");

    for i in 0..5u32 {
        let w = 64u32;
        let h = 64u32;
        let data = vec![(i * 50) as u8; (w * h * 3) as usize];
        let msg = ImageMsg::new(w, h, data);
        pub_.publish(msg).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    Ok(())
}
