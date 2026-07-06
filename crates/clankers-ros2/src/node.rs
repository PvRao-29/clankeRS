use std::marker::PhantomData;
use std::sync::Arc;

use tokio::sync::broadcast;

use clankers_core::RobotResult;

use crate::message::RosMessage;
use crate::qos::QosProfile;
use crate::sim::SimBus;

/// ROS 2 robot node handle.
pub struct RobotNode {
    pub name: String,
    bus: Arc<SimBus>,
}

impl RobotNode {
    pub async fn new(name: impl Into<String>) -> RobotResult<Self> {
        let name = name.into();
        tracing::info!(node = %name, "starting clankeRS node (sim backend)");
        Ok(Self {
            name,
            bus: SimBus::global(),
        })
    }

    pub async fn subscribe<M: RosMessage>(
        &self,
        topic: &str,
        _qos: QosProfile,
    ) -> RobotResult<Subscriber<M>> {
        let rx = self.bus.subscribe(topic).await;
        Ok(Subscriber {
            topic: topic.to_string(),
            rx,
            _marker: PhantomData,
        })
    }

    pub async fn publish<M: RosMessage>(
        &self,
        topic: &str,
        _qos: QosProfile,
    ) -> RobotResult<Publisher<M>> {
        let tx = self.bus.register_publisher(topic).await;
        Ok(Publisher {
            topic: topic.to_string(),
            tx,
            _marker: PhantomData,
        })
    }
}

pub struct Subscriber<M> {
    topic: String,
    rx: broadcast::Receiver<(u64, Vec<u8>)>,
    _marker: PhantomData<M>,
}

impl<M: RosMessage> Subscriber<M> {
    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub async fn next(&mut self) -> Option<M> {
        loop {
            match self.rx.recv().await {
                Ok((_stamp, data)) => match M::deserialize(&data) {
                    Ok(msg) => return Some(msg),
                    Err(e) => tracing::warn!(error = %e, "failed to deserialize message"),
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(dropped = n, topic = %self.topic, "subscriber lagged");
                }
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }
}

pub struct Publisher<M> {
    topic: String,
    tx: broadcast::Sender<(u64, Vec<u8>)>,
    _marker: PhantomData<M>,
}

impl<M: RosMessage> Publisher<M> {
    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub async fn publish(&self, msg: M) -> RobotResult<()> {
        let stamp = clankers_core::Timestamp::now().as_nanos();
        let data = msg.serialize();
        // Publishing with zero subscribers is valid in ROS; ignore SendError.
        let _ = self.tx.send((stamp, data));
        Ok(())
    }
}

/// Inject a message into the sim bus (used by replay).
pub async fn inject_message<M: RosMessage>(topic: &str, msg: M) -> RobotResult<()> {
    let bus = SimBus::global();
    let tx = bus.register_publisher(topic).await;
    let stamp = clankers_core::Timestamp::now().as_nanos();
    let _ = tx.send((stamp, msg.serialize()));
    Ok(())
}
