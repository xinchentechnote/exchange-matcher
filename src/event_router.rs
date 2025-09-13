use std::sync::Arc;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::types::{EngineCommand, EngineEvent, L1MarketData, MatchEvent, RbCmd};

pub struct EventRouter {
    cmd_tx: UnboundedSender<EngineCommand>,
}

impl EventRouter {
    pub fn new(cmd_tx: UnboundedSender<EngineCommand>) -> Self {
        Self { cmd_tx }
    }

    pub fn on_market_data(&mut self, _md: &L1MarketData) {
        // Handle market data event
        println!("Received market data{:?}", _md);
    }

    pub async fn on_cmd(&self, _cmd: RbCmd) {
        // Handle command event
        println!("Received command{:?}", _cmd);
        let _ = self.cmd_tx.send(EngineCommand::NewOrder(_cmd));
    }

    pub async fn start(self: Arc<Self>, mut event_rx: UnboundedReceiver<EngineEvent>) {
        while let Some(event) = event_rx.recv().await {
            match event {
                EngineEvent::MatchEvent(me) => {
                    self.on_match_event(&me);
                }
            }
        }
    }

    pub fn on_match_event(&self, _me: &MatchEvent) {
        // Handle match event
        println!("Received match event{:?}", _me);
    }
}
