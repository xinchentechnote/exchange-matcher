use std::sync::Arc;

use exchange_matcher::{
    engine::match_engine::MatchEngine,
    event_router::EventRouter,
    interface::channel::{AcceptorChannel, TcpAcceptorChannel},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (cmd_tx, cmd_rx) =tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();

    let router = Arc::new(EventRouter::new(cmd_tx.clone()));

    let engine = Arc::new(tokio::sync::Mutex::new(MatchEngine::new(cmd_rx, event_tx)));

    {
        let engine_clone = engine.clone();
        tokio::spawn(async move {
            engine_clone.lock().await.start().await;
        });
    }
    println!("Match engine started.");
    {
        let router_clone = router.clone();
        tokio::spawn(async move {
            router_clone.start(event_rx).await;
        });
    }
    println!("Event router started.");
    let mut channel = TcpAcceptorChannel::new(9010, router.clone());
    let _ = channel.start().await;

    tokio::signal::ctrl_c().await.unwrap();
    println!("Shutting down...");
    Ok(())
}
