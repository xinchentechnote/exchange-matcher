mod engine;
mod types;

use std::sync::{Arc, Mutex};

use engine::matching::AutoMatchingEngine;
use engine::order_book::OrderBook;
use exchange_matcher::protocol::{FrameDecoder, SseDecoder};
use sse_binary::sse_binary::SseBinaryBodyEnum;
use tokio::{io::AsyncReadExt, net::TcpListener};
use types::{OrderRequest, OrderSide};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let order_book = Arc::new(Mutex::new(OrderBook::new()));
    let engine = AutoMatchingEngine::new(order_book.clone());
    let listener = TcpListener::bind("127.0.0.1:9010").await?;
    println!("Listening on {}", listener.local_addr()?);
    loop {
        let (mut stream, addr) = listener.accept().await?;
        println!("Accepted connection from {:?}", addr);
        let order_tx = engine.get_order_sender();
        let mut decoder = FrameDecoder::new(SseDecoder);
        tokio::spawn(async move {
            loop {
                let mut temp = [0u8; 1024];
                match stream.read(&mut temp).await {
                    Ok(0) => {
                        println!("client disconnected");
                        break;
                    }
                    Ok(n) => {
                        decoder.feed(&temp[..n]);
                    }
                    Err(e) => {
                        eprintln!("Read error: {:?}", e);
                        break;
                    }
                }

                if let Some(msg) = decoder.next_frame() {
                    match msg.body {
                        SseBinaryBodyEnum::NewOrderSingle(order) => {
                            let order_request = order.clone().into();
                            println!("order: {:?}", order);
                            let _ = order_tx.send(order_request);
                        }
                        _ => {
                            println!("unknown message type");
                        }
                    }
                }
            }
        });
    }
}
