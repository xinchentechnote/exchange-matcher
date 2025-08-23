use exchange_matcher::engine::match_engine::MatchEngine;

use exchange_matcher::interface::channel::{AcceptorChannel, TcpAcceptorChannel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = MatchEngine::new();
    let mut acceptor = TcpAcceptorChannel::new(9010, engine);
    acceptor.start().await?;
    tokio::signal::ctrl_c().await?;
    println!("Received shutdown signal, gracefully shutting down...");
    acceptor.stop().await?;
    Ok(())
}
