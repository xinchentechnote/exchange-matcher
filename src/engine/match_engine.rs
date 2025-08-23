use std::collections::HashMap;

use crate::order_book::OrderBook;
use crate::types::RbCmd;

pub struct MatchEngine {
    order_book_map: HashMap<String, OrderBook>,
}

impl MatchEngine {
    pub fn new() -> Self {
        Self {
            order_book_map: HashMap::new(),
        }
    }

    pub fn get_order_book(&mut self, security_id: String) -> &mut OrderBook {
        self.order_book_map
            .entry(security_id.to_string())
            .or_insert_with(|| OrderBook::new(security_id))
    }

    pub fn match_order(&mut self, cmd: &mut RbCmd) {
        let order_book = self.get_order_book(cmd.security_id.clone());
        order_book.new_order(cmd);
    }
}
