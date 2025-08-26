use std::{io::Error, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    engine::match_engine::MatchEngine,
    interface::channel::{AcceptorChannel, TcpAcceptorChannel},
    types::{L1MarketData, MatchEvent, RbCmd},
};

#[async_trait::async_trait]
pub trait MatcherApp: Send + Sync {
    async fn init(self_arc: Arc<Mutex<Self>>)
    where
        Self: Sized;
    async fn start(&mut self) -> Result<(), Error>;
    async fn on_cmd(&mut self, cmd: &mut RbCmd);
    fn on_match_event(&mut self, me: &MatchEvent);
    fn on_market_data(&mut self, md: &L1MarketData);

    async fn stop(&mut self) -> Result<(), Error>;
}
pub struct DefaultMatcherApp {
    pub engine: Arc<Mutex<MatchEngine>>,
    pub trade_channel: TcpAcceptorChannel,
    pub self_ref: Option<Arc<Mutex<dyn MatcherApp + Send>>>,
}

impl DefaultMatcherApp {
    pub fn new(port: u16) -> Arc<Mutex<Self>> {
        let engine = Arc::new(Mutex::new(MatchEngine::new()));
        Arc::new(Mutex::new(Self {
            engine: engine.clone(),
            trade_channel: TcpAcceptorChannel::new(port),
            self_ref: None,
        }))
    }
}

#[async_trait]
impl MatcherApp for DefaultMatcherApp {
    async fn init(self_arc: Arc<Mutex<Self>>) {
        {
            let mut app = self_arc.lock().await;
            app.self_ref = Some(self_arc.clone() as Arc<Mutex<dyn MatcherApp + Send>>);
        }

        let self_ref = {
            let app = self_arc.lock().await;
            app.self_ref.as_ref().unwrap().clone()
        };

        self_arc
            .lock()
            .await
            .engine
            .lock()
            .await
            .set_app(Arc::downgrade(&self_ref));

        self_arc
            .lock()
            .await
            .trade_channel
            .set_app(Arc::downgrade(&self_ref));
    }

    async fn start(&mut self) -> Result<(), Error> {
        self.trade_channel.start().await
    }

    async fn on_cmd(&mut self, cmd: &mut RbCmd) {
        let mut engine = self.engine.lock().await;
        println!("on_cmd:{:?}", &cmd);
        engine.match_order(cmd).await;
    }

    fn on_match_event(&mut self, me: &MatchEvent) {
        println!("on_match_event:{:?}", me);
    }

    fn on_market_data(&mut self, md: &L1MarketData) {
        println!("on_market_data:{:?}", md);
    }

    async fn stop(&mut self) -> Result<(), Error> {
        self.trade_channel.stop().await
    }
}
