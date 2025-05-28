use ordered_float::OrderedFloat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderRequest {
    pub id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub price: OrderedFloat<f64>,
    pub qty: u64,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeReport {
    pub order_id: String,
    pub counter_order_id: String,
    pub symbol: String,
    pub price: OrderedFloat<f64>,
    pub qty: u64,
    pub trade_time: u64,
}
