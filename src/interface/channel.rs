use sse_binary::sse_binary::SseBinaryBodyEnum;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;
use tracing::info;

use crate::interface::channel;
use crate::protocol::proto::FrameDecoder;
use crate::protocol::proto::SseDecoder;
use crate::types::EngineCommand;
use crate::types::EngineEvent;
use crate::types::Order;
use crate::types::RbCmd;

pub trait AcceptorChannel {
    fn start(
        &mut self,
        event_rx: UnboundedReceiver<EngineEvent>,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn stop(&mut self) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}

#[derive(Clone)]
pub struct TcpAcceptorChannel {
    port: u16,
    running: bool,
    cmd_tx: UnboundedSender<EngineCommand>,
}

impl TcpAcceptorChannel {
    pub fn new(port: u16, cmd_tx: UnboundedSender<EngineCommand>) -> Self {
        Self {
            port: port,
            running: false,
            cmd_tx,
        }
    }
}

impl AcceptorChannel for TcpAcceptorChannel {
    async fn start(&mut self, mut event_rx: UnboundedReceiver<EngineEvent>) -> Result<(), Error> {
        if self.running {
            return Err(Error::new(ErrorKind::Other, "Acceptor already running"));
        }

        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        info!("Listening on {}", listener.local_addr()?);

        let channel = Arc::new(self.clone());
        self.running = true;

        let c = channel.clone();
        tokio::spawn(async move {
            match c.run_acceptor(listener).await {
                Ok(_) => info!("Acceptor stopped normally"),
                Err(e) => error!("Acceptor error: {}", e),
            }
        });

        tokio::spawn(async move {
            //self.event_rx
            while let Some(event) = event_rx.recv().await {
                match event {
                    EngineEvent::MatchEvent(me) => {
                        info!("Match Event: {:?}", me);
                    }
                }
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
    async fn run_acceptor(self: Arc<Self>, listener: TcpListener) -> Result<(), Error> {
        loop {
            let (stream, addr) = listener.accept().await?;
            let channel = self.clone();
            info!("Accepted connection from {:?}", addr);
            tokio::spawn(async move {
                if let Err(e) = channel.handle_connection(stream).await {
                    error!("Connection handler error: {}", e);
                }
            });
        }
    }

    async fn handle_connection(self: Arc<Self>, mut stream: TcpStream) -> Result<(), Error> {
        let mut decoder = FrameDecoder::new(SseDecoder);
        let mut buffer = [0u8; 1024];

        loop {
            let n = stream.read(&mut buffer).await?;
            if n == 0 {
                info!("Client disconnected");
                return Ok(());
            }

            decoder.feed(&buffer[..n]);
            while let Some(msg) = decoder.next_frame() {
                match msg.body {
                    SseBinaryBodyEnum::Logon(logon) => {
                        info!("Logon received: {:?}", logon);
                    }
                    SseBinaryBodyEnum::Heartbeat(_) => {
                        info!("Heartbeat received");
                    }
                    SseBinaryBodyEnum::NewOrderSingle(order) => {
                        let order_request = Order::from(&order);
                        let cmd = RbCmd {
                            side: order_request.side,
                            match_event_list: vec![],
                            price: order_request.price,
                            volume: order_request.volume,
                            mid: 0,
                            oid: order_request.oid,
                            uid: order_request.uid,
                            security_id: order_request.security_id,
                        };
                        info!("Order will process: {:?}", cmd);
                        self.cmd_tx.send(EngineCommand::NewOrder(cmd));
                    }
                    _ => {
                        info!("Unknown message type received");
                    }
                }
            }
        }
    }
}
