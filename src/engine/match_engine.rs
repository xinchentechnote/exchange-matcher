use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::event_router::EventRouter;
use crate::order_book::OrderBook;
use crate::types::RbCmd;

pub struct MatchEngine {
    order_book_map: HashMap<String, OrderBook>,
    router: Arc<Mutex<EventRouter>>,
}

impl MatchEngine {
    pub fn new(router: Arc<Mutex<EventRouter>>) -> Self {
        Self {
            order_book_map: HashMap::new(),
            router: router,
        }
    }
    pub fn get_order_book(&mut self, security_id: String) -> &mut OrderBook {
        self.order_book_map
            .entry(security_id.to_string())
            .or_insert_with(|| OrderBook::new(security_id))
    }

    pub async fn match_order(&mut self, cmd: &mut RbCmd) {
        let order_book = self.get_order_book(cmd.security_id.clone());
        order_book.new_order(cmd);
        if cmd.match_event_list.len() > 0 {
            let router = self.router.clone();
            let mut router = router.lock().await;
            for event in cmd.match_event_list.iter() {
                router.on_match_event(event);
            }
        }
    }
}
