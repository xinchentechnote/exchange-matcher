mod engine;
mod types;

use std::sync::{Arc, Mutex};

use engine::matching::AutoMatchingEngine;
use engine::order_book::OrderBook;
use types::{OrderRequest, OrderSide};

fn main() {
    let order_book = Arc::new(Mutex::new(OrderBook::new()));
    let engine = AutoMatchingEngine::new(order_book.clone());

    // 示例订单提交
    let buy_order = OrderRequest {
        id: "1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        price: ordered_float::OrderedFloat(100.0),
        qty: 10,
        source: "test".to_string(),
    };

    let sell_order = OrderRequest {
        id: "2".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Sell,
        price: ordered_float::OrderedFloat(99.0),
        qty: 5,
        source: "test".to_string(),
    };

    println!("[MAIN] Submitting buy and sell orders...");

    engine.submit_order(buy_order);
    engine.submit_order(sell_order);

    std::thread::sleep(std::time::Duration::from_secs(1)); // 等待撮合完成
}
