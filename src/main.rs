use std::sync::Arc;

use exchange_matcher::{
    engine::match_engine::MatchEngine, event_router::EventRouter,
    interface::channel::{AcceptorChannel, TcpAcceptorChannel},
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Arc::new(Mutex::new(EventRouter::new()));
    let channel = Arc::new(Mutex::new(TcpAcceptorChannel::new(9010, router.clone())));
    let engine = Arc::new(Mutex::new(MatchEngine::new(router.clone())));
    {
        let mut r = router.lock().await;
        r.register_trade_channel(channel.clone());
        r.register_engine(engine.clone());
    }
    channel.lock().await.start().await;
    tokio::signal::ctrl_c().await.unwrap();
    println!("Shutting down...");
    Ok(())
}
