use exchange_matcher::matcher_app::{DefaultMatcherApp, MatcherApp};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = DefaultMatcherApp::new(9010);
    DefaultMatcherApp::init(app.clone()).await;
    {
        let mut app_guard = app.lock().await;
        app_guard.start().await?;
    }
    print!("Matcher started");
    tokio::signal::ctrl_c().await?;
    println!("Received shutdown signal, gracefully shutting down...");
    {
        let mut app_guard = app.lock().await;
        app_guard.stop().await?;
    }
    Ok(())
}
