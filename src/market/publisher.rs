pub trait MarketPublisher {
    fn publish_trade(&self, trade: &TradeReport);
    fn publish_snapshot(&self, snapshot: &OrderBookSnapshot);
}
