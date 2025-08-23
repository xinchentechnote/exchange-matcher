#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L1MarketData {
    pub security_id: String,
    pub new_price: i64,
    pub buy_size: usize,
    pub sell_size: usize,
    pub buy_prices: Vec<i64>,
    pub buy_volumes: Vec<i64>,
    pub sell_prices: Vec<i64>,
    pub sell_volumes: Vec<i64>,
    pub timestamp: i64,
}

impl L1MarketData {
    pub const L1_SIZE: usize = 5;

    pub fn new_with_arrays(
        buy_prices: Vec<i64>,
        buy_volumes: Vec<i64>,
        sell_prices: Vec<i64>,
        sell_volumes: Vec<i64>,
    ) -> Self {
        let buy_size = buy_prices.len();
        let sell_size = sell_prices.len();
        Self {
            security_id: "".to_string(),
            new_price: 0,
            buy_size,
            sell_size,
            buy_prices,
            buy_volumes,
            sell_prices,
            sell_volumes,
            timestamp: 0,
        }
    }

    pub fn new(buy_size: usize, sell_size: usize) -> Self {
        Self {
            security_id: "".to_string(),
            new_price: 0,
            buy_size,
            sell_size,
            buy_prices: vec![0; buy_size],
            buy_volumes: vec![0; buy_size],
            sell_prices: vec![0; sell_size],
            sell_volumes: vec![0; sell_size],
            timestamp: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdResultCode {
    Success,
    DuplicateOrderId,
    InvalidOrderId,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    OrderEd,
    TradeEd,
    PartTrade,
    CancelEd,
    PartCancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub oid: i64,
    pub mid: i64,
    pub uid: u64,
    pub security_id: String,
    pub side: OrderSide,
    pub price: i64,
    pub volume: i64,
    pub tvolume: i64,
    pub timestamp: i64,
}

impl Order {
    pub fn remaining(&self) -> i64 {
        self.volume - self.tvolume
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchEvent {
    pub timestamp: i64,
    pub mid: i64,
    pub oid: i64,
    pub status: OrderStatus,
    pub tid: i64,
    pub volume: i64,
    pub price: i64,
}
impl MatchEvent {
    pub fn default() -> MatchEvent {
        MatchEvent {
            timestamp: 0,
            mid: 0,
            oid: 0,
            status: OrderStatus::OrderEd,
            tid: 0,
            volume: 0,
            price: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RbCmd {
    pub side: OrderSide,
    pub match_event_list: Vec<MatchEvent>,
    pub price: i64,
    pub volume: i64,
    pub mid: i64,
    pub uid: u64,
    pub oid: i64,
    pub security_id: String,
}
