use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use tokio::sync::{broadcast, RwLock};

static GLOBAL_BUS: OnceLock<Arc<SimBus>> = OnceLock::new();

type MessageSender = broadcast::Sender<(u64, Vec<u8>)>;

/// In-memory pub/sub bus for simulation and replay without ROS 2 installed.
pub struct SimBus {
    topics: RwLock<HashMap<String, MessageSender>>,
}

impl SimBus {
    pub fn global() -> Arc<Self> {
        GLOBAL_BUS
            .get_or_init(|| {
                Arc::new(Self {
                    topics: RwLock::new(HashMap::new()),
                })
            })
            .clone()
    }

    pub async fn register_publisher(&self, topic: &str) -> MessageSender {
        let mut topics = self.topics.write().await;
        topics
            .entry(topic.to_string())
            .or_insert_with(|| broadcast::channel(256).0)
            .clone()
    }

    pub async fn subscribe(&self, topic: &str) -> broadcast::Receiver<(u64, Vec<u8>)> {
        let tx = self.register_publisher(topic).await;
        tx.subscribe()
    }

    pub async fn reset() {
        if let Some(bus) = GLOBAL_BUS.get() {
            let mut topics = bus.topics.write().await;
            topics.clear();
        }
    }
}
