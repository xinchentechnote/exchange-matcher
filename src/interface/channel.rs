use dashmap::DashMap;
use sse_binary::sse_binary::SseBinaryBodyEnum;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;
use tracing::info;

use crate::protocol::proto::FrameDecoder;
use crate::protocol::proto::SseDecoder;
use crate::types::EngineCommand;
use crate::types::EngineEvent;
use crate::types::Order;
use crate::types::RbCmd;

pub trait AcceptorChannel {
    fn start(
        self: Arc<Self>,
        event_rx: UnboundedReceiver<EngineEvent>,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn stop(&mut self) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}

struct Session {
    id: u64,
    writer: OwnedWriteHalf,
    tx: UnboundedSender<EngineEvent>,
}

pub struct TcpAcceptorChannel {
    port: u16,
    cmd_tx: UnboundedSender<EngineCommand>,
    session_map: Arc<DashMap<u64, Session>>,
    next_id: AtomicU64,
}

impl TcpAcceptorChannel {
    pub fn new(port: u16, cmd_tx: UnboundedSender<EngineCommand>) -> Arc<Self> {
        Arc::new(Self {
            port: port,
            cmd_tx,
            session_map: Arc::new(DashMap::new()),
            next_id: AtomicU64::new(1),
        })
    }

    pub fn next_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

impl AcceptorChannel for TcpAcceptorChannel {
    async fn start(
        self: Arc<Self>,
        mut event_rx: UnboundedReceiver<EngineEvent>,
    ) -> Result<(), Error> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        info!("Listening on {}", listener.local_addr()?);

        let c = self.clone();
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
                        if let Some(session_ref) = self.session_map.get(&me.session_id) {
                            if let Err(e) = session_ref.tx.send(EngineEvent::MatchEvent(me.clone()))
                            {
                                error!(
                                    "Failed to send MatchEvent to session {}: {}",
                                    me.session_id, e
                                );
                            }
                        } else {
                            info!("Session {} not found, maybe disconnected", me.session_id);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl TcpAcceptorChannel {
    async fn run_acceptor(self: Arc<Self>, listener: TcpListener) -> Result<(), Error> {
        loop {
            let (stream, addr) = listener.accept().await?;
            info!("Accepted connection from {:?}", addr);
            let channel = self.clone();
            if let Err(e) = channel.handle_connection(stream, addr).await {
                error!("Connection handler error: {}", e);
            }
        }
    }

    async fn handle_connection(
        self: Arc<Self>,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), Error> {
        let (reader, writer) = stream.into_split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let session_id = self.next_id();
        let session = Session {
            id: session_id,
            writer: writer,
            tx: tx,
        };
        self.session_map.insert(session_id, session);
        let mut decoder = FrameDecoder::new(SseDecoder);
        let mut buffer = [0u8; 1024];
        tokio::spawn(async move {
            let mut reader = BufReader::new(reader);
            loop {
                let n = reader.read(&mut buffer).await.unwrap();
                if n == 0 {
                    info!("Client disconnected");
                    break;
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
                                session_id,
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
                            let _ = self.cmd_tx.send(EngineCommand::NewOrder(cmd));
                        }
                        _ => {
                            info!("Unknown message type received");
                        }
                    }
                }
            }
        });
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    EngineEvent::MatchEvent(me) => {
                        info!("Sending Match Event to client {}: {:?}", addr, me);
                    }
                }
            }
        });

        Ok(())
    }
}
