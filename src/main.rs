mod engine;
mod interface;
mod protocol;
mod types;

use std::sync::{Arc, Mutex};

use engine::matching::AutoMatchingEngine;
use engine::order_book::OrderBook;

use crate::interface::channel::{AcceptorChannel, TcpAcceptorChannel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let order_book = Arc::new(Mutex::new(OrderBook::new()));
    let engine = AutoMatchingEngine::new(order_book.clone());
    let mut acceptor = TcpAcceptorChannel::new(9010, engine.get_order_sender());
    acceptor.start().await?;
    tokio::signal::ctrl_c().await?;
    println!("Received shutdown signal, gracefully shutting down...");

    acceptor.stop().await?;
    Ok(())
}
