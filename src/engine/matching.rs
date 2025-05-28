use crossbeam_channel::{Receiver, Sender, unbounded};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::engine::order_book::OrderBook;
use crate::types::OrderRequest;

pub struct AutoMatchingEngine {
    order_sender: Sender<OrderRequest>,
}

impl AutoMatchingEngine {
    pub fn new(order_book: Arc<Mutex<OrderBook>>) -> Self {
        let (tx, rx): (Sender<OrderRequest>, Receiver<OrderRequest>) = unbounded();

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

    pub fn submit_order(&self, order: OrderRequest) {
        let _ = self.order_sender.send(order);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OrderRequest, OrderSide};
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
        engine.submit_order(OrderRequest {
            id: "1".to_string(),
            side: OrderSide::Buy,
            price: ordered_float::OrderedFloat(100.0),
            qty: 10,
            symbol: "BTCUSDT".to_string(),
            source: "test".to_string(),
        });

        // 提交一个卖单
        engine.submit_order(OrderRequest {
            id: "2".to_string(),
            side: OrderSide::Sell,
            price: ordered_float::OrderedFloat(100.0),
            qty: 10,
            symbol: "BTCUSDT".to_string(),
            source: "test".to_string(),
        });

        // 等待撮合线程运行
        thread::sleep(Duration::from_millis(500));

        // 验证订单薄应为空
        let book = order_book.lock().unwrap();
        assert!(book.bids.is_empty(), "Bids should be empty after match");
        assert!(book.asks.is_empty(), "Asks should be empty after match");
    }
}
