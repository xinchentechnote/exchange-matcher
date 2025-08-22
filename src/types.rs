use ordered_float::OrderedFloat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub member_id: u16,
    pub uid: u64,
    pub security_id: String,
    pub side: OrderSide,
    pub price: u64,
    pub qty: u64,
    pub traded_qty: u64,
    pub order_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub timestamp: i64,
    pub member_id: u16,
    pub order_id: String,
    pub order_status: u16,
    pub price: u64,
    pub qty: u64,
}
