use sse_binary::sse_binary::SseBinaryBodyEnum;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::event_router::EventRouter;
use crate::protocol::proto::FrameDecoder;
use crate::protocol::proto::SseDecoder;
use crate::types::Order;
use crate::types::RbCmd;

pub trait AcceptorChannel {
    async fn start(&mut self) -> Result<(), Error>;
    async fn stop(&mut self) -> Result<(), Error>;
}

pub struct TcpAcceptorChannel {
    tcp_listener: Option<TcpListener>,
    decoder: Option<FrameDecoder<SseDecoder>>,
    router: Arc<Mutex<EventRouter>>,
    port: u16,
    running: bool,
}

impl TcpAcceptorChannel {
    pub fn new(port: u16, router: Arc<Mutex<EventRouter>>) -> Self {
        Self {
            tcp_listener: None,
            port: port,
            decoder: None,
            running: false,
            router,
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
        if let router = self.router.clone() {
            tokio::spawn(async move {
                match Self::run_acceptor(listener, router).await {
                    Ok(_) => println!("Acceptor stopped normally"),
                    Err(e) => eprintln!("Acceptor error: {}", e),
                }
            });
        }

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
        router: Arc<Mutex<EventRouter>>,
    ) -> Result<(), Error> {
        loop {
            let (mut stream, addr) = listener.accept().await?;
            println!("Accepted connection from {:?}", addr);
            let router = router.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, router).await {
                    eprintln!("Connection handler error: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        router: Arc<Mutex<EventRouter>>,
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
                    SseBinaryBodyEnum::Logon(logon) => {
                        println!("Logon received: {:?}", logon);
                    }
                    SseBinaryBodyEnum::Heartbeat(_) => {
                        println!("Heartbeat received");
                    }
                    SseBinaryBodyEnum::NewOrderSingle(order) => {
                        let order_request = Order::from(&order);
                        let mut cmd = RbCmd {
                            side: order_request.side,
                            match_event_list: vec![],
                            price: order_request.price,
                            volume: order_request.volume,
                            mid: 0,
                            oid: order_request.oid,
                            uid: order_request.uid,
                            security_id: order_request.security_id,
                        };
                        print!("Order will process: {:?}", cmd);
                        let mut r = router.lock().await;
                        r.on_cmd(&mut cmd);
                        println!("Order received: {:?}", order);
                        print!("Order processed: {:?}", cmd)
                    }
                    _ => {
                        println!("Unknown message type received");
                    }
                }
            }
        }
    }
}
