use std::sync::RwLock;

use protocol::tokio::{self};

pub enum StopOpt {
    MineNBlocks(u64),
    MineToHeight(u64),
}

type SignalSender = tokio::sync::oneshot::Sender<()>;

pub struct StopSignal {
    tx:             RwLock<Option<SignalSender>>,
    stop_at_height: Option<u64>,
}

impl StopSignal {
    pub fn new(tx: SignalSender) -> Self {
        Self {
            tx:             RwLock::new(Some(tx)),
            stop_at_height: None,
        }
    }

    pub fn with_stop_at(tx: SignalSender, height: u64) -> Self {
        Self {
            tx:             RwLock::new(Some(tx)),
            stop_at_height: Some(height),
        }
    }

    pub fn check_height_and_send(&self, height: u64) {
        if Some(height) == self.stop_at_height {
            self.send();
        }
    }

    pub fn send(&self) {
        if let Some(tx) = self.tx.write().unwrap().take() {
            let _ = tx.send(());
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.tx.read().unwrap().is_none()
    }
}
