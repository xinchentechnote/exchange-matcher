use crossbeam_channel::{Receiver, Sender, unbounded};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::engine::order_book::OrderBook;
use crate::types::Order;

pub trait MatchingEngine {}

pub struct AutoMatchingEngine {
    order_sender: Sender<Order>,
}

impl AutoMatchingEngine {
    pub fn get_order_sender(&self) -> Sender<Order> {
        self.order_sender.clone()
    }

    pub fn new(order_book: Arc<Mutex<OrderBook>>) -> Self {
        let (tx, rx): (Sender<Order>, Receiver<Order>) = unbounded();

        // 撮合线程
        let order_book_clone = Arc::clone(&order_book);
        thread::spawn(move || {
            loop {
                if let Ok(order) = rx.try_recv() {
                    let mut book = order_book_clone.lock().unwrap();
                    let results = book.match_order(order);
                    for result in results {
                        println!("[MATCH RESULT] {:?}", result);
                    }
                }

                thread::sleep(Duration::from_millis(100)); // 每 100ms 检查一次
            }
        });

        Self { order_sender: tx }
    }

    pub fn submit_order(&self, order: Order) {
        let _ = self.order_sender.send(order);
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::types::{Order, OrderSide};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_auto_matching_engine_matches_orders() {
        // 初始化共享 OrderBook
        let order_book = Arc::new(Mutex::new(OrderBook::new()));

        // 创建引擎
        let engine = AutoMatchingEngine::new(order_book.clone());

        // 提交一个买单
        engine.submit_order(Order {
            order_id: "9527".to_string(),
            side: OrderSide::Buy,
            price: 100,
            qty: 10,
            security_id: "000001".to_string(),
            member_id: 9527,
            uid: 9527,
            traded_qty: 0,
            timestamp: Utc::now().timestamp_millis(),
        });

        // 提交一个卖单
        engine.submit_order(Order {
            order_id: "9528".to_string(),
            side: OrderSide::Sell,
            price: 100,
            qty: 10,
            security_id: "000001".to_string(),
            member_id: 9527,
            uid: 9527,
            traded_qty: 0,
            timestamp: Utc::now().timestamp_millis(),
        });

        // 等待撮合线程运行
        thread::sleep(Duration::from_millis(1000));

        // 验证订单薄应为空
        let book = order_book.lock().unwrap();
        assert!(book.bids.is_empty(), "Bids should be empty after match");
        assert!(book.asks.is_empty(), "Asks should be empty after match");
    }
}
