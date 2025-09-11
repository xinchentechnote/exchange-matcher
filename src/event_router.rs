use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    engine::match_engine::MatchEngine,
    interface::channel::TcpAcceptorChannel,
    types::{L1MarketData, MatchEvent, RbCmd},
};

pub struct EventRouter {
    pub engine: Option<Arc<Mutex<MatchEngine>>>,
    pub trade_channel: Option<Arc<Mutex<TcpAcceptorChannel>>>,
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            engine: None,
            trade_channel: None,
        }
    }
    pub fn register_engine(&mut self, engine: Arc<Mutex<MatchEngine>>) {
        self.engine = Some(engine);
    }

    pub fn register_trade_channel(&mut self, trade_channel: Arc<Mutex<TcpAcceptorChannel>>) {
        self.trade_channel = Some(trade_channel);
    }

    pub fn on_market_data(&mut self, _md: &L1MarketData) {
        // Handle market data event
    }

    pub fn on_cmd(&mut self, _cmd: &mut RbCmd) {
        // Handle command event
    }

    pub fn on_match_event(&mut self, _me: &MatchEvent) {
        // Handle match event
    }
}
