//! Tests for the in-memory sim pub/sub backend.
//!
//! The sim bus is process-global, so each test uses a unique topic name to
//! avoid cross-talk when tests run in parallel.

use clankers_ros2::{inject_message, ImageMsg, QosProfile, RobotNode};

#[tokio::test]
async fn publish_then_receive() {
    let node = RobotNode::new("sim_deliver").await.unwrap();
    // Subscribe before publishing: the broadcast bus only delivers messages
    // sent after the subscription exists.
    let mut sub = node
        .subscribe::<ImageMsg>("/test/deliver", QosProfile::default())
        .await
        .unwrap();
    let publisher = node
        .publish::<ImageMsg>("/test/deliver", QosProfile::default())
        .await
        .unwrap();

    publisher
        .publish(ImageMsg::new(8, 8, vec![3u8; 8 * 8 * 3]))
        .await
        .unwrap();

    let got = sub.next().await.unwrap();
    assert_eq!(got.width, 8);
    assert_eq!(got.height, 8);
    assert_eq!(sub.topic(), "/test/deliver");
}

#[tokio::test]
async fn publish_with_no_subscriber_is_ok() {
    let node = RobotNode::new("sim_nosub").await.unwrap();
    let publisher = node
        .publish::<ImageMsg>("/test/no_subscriber", QosProfile::default())
        .await
        .unwrap();
    // No subscriber -> still succeeds (matches ROS semantics).
    assert!(publisher
        .publish(ImageMsg::new(2, 2, vec![0u8; 12]))
        .await
        .is_ok());
}

#[tokio::test]
async fn inject_message_reaches_subscriber() {
    let node = RobotNode::new("sim_inject").await.unwrap();
    let mut sub = node
        .subscribe::<ImageMsg>("/test/inject", QosProfile::default())
        .await
        .unwrap();

    inject_message(
        "/test/inject",
        ImageMsg::new(16, 16, vec![9u8; 16 * 16 * 3]),
    )
    .await
    .unwrap();

    let got = sub.next().await.unwrap();
    assert_eq!(got.width, 16);
}

#[tokio::test]
async fn multiple_subscribers_all_receive() {
    let node = RobotNode::new("sim_multi").await.unwrap();
    let mut sub_a = node
        .subscribe::<ImageMsg>("/test/multi", QosProfile::default())
        .await
        .unwrap();
    let mut sub_b = node
        .subscribe::<ImageMsg>("/test/multi", QosProfile::default())
        .await
        .unwrap();
    let publisher = node
        .publish::<ImageMsg>("/test/multi", QosProfile::default())
        .await
        .unwrap();

    publisher
        .publish(ImageMsg::new(4, 4, vec![1u8; 48]))
        .await
        .unwrap();

    assert_eq!(sub_a.next().await.unwrap().width, 4);
    assert_eq!(sub_b.next().await.unwrap().width, 4);
}
