use std::collections::HashMap;
use std::sync::{Arc, Weak};

use tokio::sync::Mutex;

use crate::matcher_app::MatcherApp;
use crate::order_book::OrderBook;
use crate::types::RbCmd;

pub struct MatchEngine {
    order_book_map: HashMap<String, OrderBook>,
    app: Option<Weak<Mutex<dyn MatcherApp + Send>>>,
}

impl MatchEngine {
    pub fn new() -> Self {
        Self {
            order_book_map: HashMap::new(),
            app: None,
        }
    }

    pub fn set_app(&mut self, app: Weak<Mutex<dyn MatcherApp + Send>>) {
        self.app = Some(app);
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
            if let Some(app) = &self.app {
                if let Some(app_arc) = app.upgrade() {
                    let mut app = app_arc.lock().await;
                    for event in cmd.match_event_list.iter() {
                        app.on_match_event(event);
                    }
                } else {
                    eprintln!("App has been dropped");
                }
            }
        }
    }
}
