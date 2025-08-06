use binary_codec::BinaryCodec;
use bytes::{Buf, Bytes, BytesMut};
use sse_binary::sse_binary::SseBinary;

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
