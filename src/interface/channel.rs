use crossbeam_channel::Sender;
use sse_binary::sse_binary::SseBinaryBodyEnum;
use std::io::Error;
use std::io::ErrorKind;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use crate::protocol::proto::FrameDecoder;
use crate::protocol::proto::SseDecoder;
use crate::types::OrderRequest;

pub trait AcceptorChannel {
    async fn start(&mut self) -> Result<(), Error>;
    async fn stop(&mut self) -> Result<(), Error>;
}

pub struct TcpAcceptorChannel {
    tcp_listener: Option<TcpListener>,
    order_sender: Sender<OrderRequest>,
    decoder: Option<FrameDecoder<SseDecoder>>,
    port: u16,
    running: bool,
}

impl TcpAcceptorChannel {
    pub fn new(port: u16, sender: Sender<OrderRequest>) -> Self {
        Self {
            tcp_listener: None,
            order_sender: sender,
            port: port,
            decoder: None,
            running: false,
        }
    }
}
impl AcceptorChannel for TcpAcceptorChannel {
    async fn start(&mut self) -> Result<(), Error> {
        if self.running {
            return Err(Error::new(ErrorKind::Other, "Acceptor already running"));
        }

        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        println!("Listening on {}", listener.local_addr()?);

        self.running = true;
        let sender = self.order_sender.clone();

        tokio::spawn(async move {
            match Self::run_acceptor(listener, sender).await {
                Ok(_) => println!("Acceptor stopped normally"),
                Err(e) => eprintln!("Acceptor error: {}", e),
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        self.running = false;
        Ok(())
    }
}

impl TcpAcceptorChannel {
    async fn run_acceptor(
        listener: TcpListener,
        sender: Sender<OrderRequest>,
    ) -> Result<(), Error> {
        loop {
            let (mut stream, addr) = listener.accept().await?;
            println!("Accepted connection from {:?}", addr);

            let order_tx = sender.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, order_tx).await {
                    eprintln!("Connection handler error: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        sender: Sender<OrderRequest>,
    ) -> Result<(), Error> {
        let mut decoder = FrameDecoder::new(SseDecoder);
        let mut buffer = [0u8; 1024];

        loop {
            let n = stream.read(&mut buffer).await?;
            if n == 0 {
                println!("Client disconnected");
                return Ok(());
            }

            decoder.feed(&buffer[..n]);

            while let Some(msg) = decoder.next_frame() {
                match msg.body {
                    SseBinaryBodyEnum::NewOrderSingle(order) => {
                        let order_request = OrderRequest::from(&order);
                        println!("Order received: {:?}", order);
                        let _ = sender.send(order_request);
                    }
                    _ => {
                        println!("Unknown message type received");
                    }
                }
            }
        }
    }
}
