use std::sync::Arc;

use tokio::sync::broadcast;

#[derive(Clone)]
pub struct BridgeBroadcast {
    tx: Arc<broadcast::Sender<String>>,
}

impl BridgeBroadcast {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx: Arc::new(tx) }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    pub fn publish(&self, json: String) {
        let _ = self.tx.send(json);
    }
}
