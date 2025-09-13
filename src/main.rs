use std::sync::Arc;

use exchange_matcher::{
    engine::match_engine::MatchEngine,
    interface::channel::{AcceptorChannel, TcpAcceptorChannel},
};
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();

    let engine = Arc::new(tokio::sync::Mutex::new(MatchEngine::new(cmd_rx, event_tx)));
    {
        let engine_clone = engine.clone();
        tokio::spawn(async move {
            engine_clone.lock().await.start().await;
        });
    }
    info!("Match engine started.");

    let mut channel = TcpAcceptorChannel::new(9010, cmd_tx);
    let _ = channel.start(event_rx).await;

    tokio::signal::ctrl_c().await.unwrap();
    info!("Shutting down...");
    Ok(())
}
