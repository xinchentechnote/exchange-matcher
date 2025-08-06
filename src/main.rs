mod engine;
mod types;

use std::{
    io::Read,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use binary_codec::BinaryCodec;
use bytes::{Buf, BytesMut};
use engine::matching::AutoMatchingEngine;
use engine::order_book::OrderBook;
use sse_binary::sse_binary::{SseBinary, SseBinaryBodyEnum};
use types::{OrderRequest, OrderSide};

fn main() {
    let order_book = Arc::new(Mutex::new(OrderBook::new()));
    let engine = AutoMatchingEngine::new(order_book.clone());
    let listener = TcpListener::bind("127.0.0.1:9010").unwrap();
    print!("Listening on {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        let order_tx = engine.get_order_sender();
        let mut stream = stream.unwrap();
        thread::spawn(move || {
            let mut buffer = BytesMut::with_capacity(4096);
            loop {
                let mut temp = [0u8; 1024];
                let bytes_read = match stream.read(&mut temp) {
                    Ok(0) => break, // 对端关闭
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Read error: {:?}", e);
                        break;
                    }
                };

                buffer.extend_from_slice(&temp[..bytes_read]);

                // 处理粘包/半包
                loop {
                    if buffer.len() < 16 {
                        // 假设头部至少 16 字节：msg_type(4) + msg_seq_num(8) + body_len(4)
                        break;
                    }

                    // 读取头部 (不消费)
                    let mut header = &buffer[..16];
                    let _msg_type = header.get_u32();
                    let _msg_seq_num = header.get_u64();
                    let msg_body_len = header.get_u32() as usize;

                    let total_len = 16 + msg_body_len + 4; // 头(16) + body + checksum(4)

                    if buffer.len() < total_len {
                        // 数据还没到齐
                        break;
                    }

                    // 拆出一个完整消息
                    let msg_bytes = buffer.split_to(total_len).freeze();
                    let mut msg_buf = msg_bytes.clone();

                    if let Some(decoded) = SseBinary::decode(&mut msg_buf) {
                        println!("[DECODED] {:?}", decoded);
                        match decoded.body {
                            SseBinaryBodyEnum::NewOrderSingle(ref order) => {
                                let order_request = OrderRequest {
                                    id: order.cl_ord_id.clone(),
                                    symbol: order.security_id.clone(),
                                    side: OrderSide::Buy,
                                    price: ordered_float::OrderedFloat(order.price as f64),
                                    qty: order.order_qty as u64,
                                    source: "manual".to_string(),
                                };
                                let _ = order_tx.send(order_request);
                            }
                            _ => {
                                println!("[UNKNOWN] {:?}", decoded);
                            }
                        }
                    } else {
                        eprintln!("Failed to decode a message");
                    }
                }
            }
        });
    }

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
