use binary_codec::BinaryCodec;
use bytes::{Buf, Bytes, BytesMut};
use chrono::Utc;
use sse_binary::{new_order_single::NewOrderSingle, report::Report, sse_binary::SseBinary};

use crate::types::{MatchEvent, Order, OrderSide};

pub trait ProtocolDecoder: Send + Sync {
    type Message;

    fn decode(&mut self, buf: &mut Bytes) -> Option<Self::Message>;
}

pub struct SseDecoder;
impl ProtocolDecoder for SseDecoder {
    type Message = SseBinary;

    fn decode(&mut self, buf: &mut Bytes) -> Option<Self::Message> {
        SseBinary::decode(buf)
    }
}

pub struct FrameDecoder<D: ProtocolDecoder> {
    buffer: BytesMut,
    decoder: D,
}

impl<D: ProtocolDecoder> FrameDecoder<D> {
    pub fn new(decoder: D) -> Self {
        Self {
            buffer: BytesMut::with_capacity(4096),
            decoder,
        }
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn next_frame(&mut self) -> Option<D::Message> {
        // 1. 判断缓冲区是否至少有头部长度
        if self.buffer.len() < 16 {
            return None;
        }

        // 2. 从缓冲区读取消息头(16字节)，但不移除数据
        let mut header = &self.buffer[..16];
        let _msg_type = header.get_u32();
        let _msg_seq_num = header.get_u64();
        let msg_body_len = header.get_u32() as usize;

        let total_len = 16 + msg_body_len + 4; // 头 + 体 + 校验

        // 3. 判断缓冲区是否包含完整消息
        if self.buffer.len() < total_len {
            return None;
        }

        // 4. 拆出完整消息字节
        let msg_bytes = self.buffer.split_to(total_len).freeze();

        // 5. 解码
        let mut msg_buf = msg_bytes.clone();
        self.decoder.decode(&mut msg_buf)
    }
}

impl From<&NewOrderSingle> for Order {
    fn from(order: &NewOrderSingle) -> Self {
        let side = match order.side.as_str() {
            "1" => OrderSide::Buy,
            _ => OrderSide::Sell,
        };

        Order {
            session_id: 0,
            oid: order.cl_ord_id.parse::<i64>().unwrap(),
            security_id: order.security_id.clone(),
            side: side,
            price: order.price,
            volume: order.order_qty,
            mid: 0,
            uid: 0,
            tvolume: 0,
            timestamp: Utc::now().timestamp_millis(),
        }
    }
}

impl From<&MatchEvent> for Report {
    fn from(me: &MatchEvent) -> Self {
        Report {
            pbu: "".to_string(),
            set_id: 1,
            report_index: 1,
            biz_id: 1,
            exec_type: "".to_string(),
            biz_pbu: "".to_string(),
            cl_ord_id: me.oid.to_string(),
            security_id: "".to_string(),
            account: "".to_string(),
            owner_type: 1,
            order_entry_time: me.timestamp as u64,
            last_px: me.price,
            last_qty: me.volume,
            gross_trade_amt: me.price * me.volume,
            side: "1".to_string(),
            order_qty: 1,
            leaves_qty: 1,
            ord_status: "".to_string(),
            credit_tag: "".to_string(),
            clearing_firm: "".to_string(),
            branch_id: "".to_string(),
            trd_cnfm_id: "".to_string(),
            ord_cnfm_id: me.tid.to_string(),
            trade_date: 1,
            transact_time: me.timestamp as u64,
            user_info: "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use sse_binary::new_order_single::NewOrderSingle;

    use super::*;
    #[test]
    fn test_order_request_from_new_order_single() {
        let order = NewOrderSingle {
            cl_ord_id: "123".to_string(),
            security_id: "AAPL".to_string(),
            side: "BUY".to_string(),
            price: 1000,
            order_qty: 100,
            biz_id: 10,
            biz_pbu: "123".to_string(),
            account: "123".to_string(),
            owner_type: 1,
            ord_type: "2".to_string(),
            time_in_force: "3".to_string(),
            transact_time: 20250101,
            credit_tag: "3".to_string(),
            clearing_firm: "3".to_string(),
            branch_id: "test".to_string(),
            user_info: "xxx".to_string(),
        };
        let order = Order::from(&order);
        assert_eq!(order.oid, 123);
    }
}
