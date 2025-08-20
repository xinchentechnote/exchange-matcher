use ordered_float::OrderedFloat;
use sse_binary::new_order_single::NewOrderSingle;

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

impl From<&NewOrderSingle> for OrderRequest {
    fn from(order: &NewOrderSingle) -> Self {
        let side = match order.side.as_str() {
            "1" => OrderSide::Buy,
            _ => OrderSide::Sell,
        };
        OrderRequest {
            id: order.cl_ord_id.clone(),
            symbol: order.security_id.clone(),
            side: side,
            price: ordered_float::OrderedFloat(order.price as f64),
            qty: order.order_qty as u64,
            source: "manual".to_string(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_order_request_from_new_order_single() {
        let order = NewOrderSingle {
            cl_ord_id: "123".to_string(),
            security_id: "AAPL".to_string(),
            side: "BUY".to_string(),
            price: 1000,
            order_qty: 100,
            biz_id: 10,
            biz_pbu: "123".to_string(),
            account: "123".to_string(),
            owner_type: 1,
            ord_type: "2".to_string(),
            time_in_force: "3".to_string(),
            transact_time: 20250101,
            credit_tag: "3".to_string(),
            clearing_firm: "3".to_string(),
            branch_id: "test".to_string(),
            user_info: "xxx".to_string(),
        };
        let order_request = OrderRequest::from(&order);
        assert_eq!(order_request.id, "123");
    }
}
