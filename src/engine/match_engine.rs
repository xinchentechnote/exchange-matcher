use std::collections::HashMap;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::order_book::OrderBook;
use crate::types::{EngineCommand, EngineEvent, RbCmd};

pub struct MatchEngine {
    order_book_map: HashMap<String, OrderBook>,
    cmd_rx: UnboundedReceiver<EngineCommand>,
    event_tx: UnboundedSender<EngineEvent>,
}

impl MatchEngine {
    pub fn new(
        cmd_rx: UnboundedReceiver<EngineCommand>,
        event_tx: UnboundedSender<EngineEvent>,
    ) -> Self {
        Self {
            order_book_map: HashMap::new(),
            cmd_rx,
            event_tx,
        }
    }
    fn get_order_book(&mut self, security_id: String) -> &mut OrderBook {
        self.order_book_map
            .entry(security_id.to_string())
            .or_insert_with(|| OrderBook::new(security_id))
    }

    fn match_order(&mut self, cmd: &mut RbCmd) {
        let order_book = self.get_order_book(cmd.security_id.clone());
        order_book.new_order(cmd);
        for event in cmd.match_event_list.iter() {
            let _ = self.event_tx.send(EngineEvent::MatchEvent(event.clone()));
        }
    }

    pub async fn start(&mut self) {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                EngineCommand::NewOrder(mut rb_cmd) => {
                    self.match_order(&mut rb_cmd);
                }
            }
        }
    }
}
