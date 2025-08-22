use crate::types::{MatchResult, Order, OrderSide};
use std::collections::{BTreeMap, VecDeque};

#[derive(Default)]
pub struct OrderBook {
    pub bids: BTreeMap<u64, VecDeque<Order>>, // price descending
    pub asks: BTreeMap<u64, VecDeque<Order>>, // price ascending
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, order: Order) {
        let book = match order.side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };
        book.entry(order.price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    pub fn match_order(&mut self, mut taker: Order) -> Vec<MatchResult> {
        let mut trades = Vec::new();
        let book = match taker.side {
            OrderSide::Buy => &mut self.asks,
            OrderSide::Sell => &mut self.bids,
        };

        let price_match = |book_price: u64| match taker.side {
            OrderSide::Buy => taker.price >= book_price,
            OrderSide::Sell => taker.price <= book_price,
        };

        let mut matched_prices: Vec<u64> = book
            .keys()
            .cloned()
            .filter(|&price| price_match(price))
            .collect();

        if taker.side == OrderSide::Buy {
            matched_prices.sort_by(|a, b| a.partial_cmp(b).unwrap()); // lowest ask first
        } else {
            matched_prices.sort_by(|a, b| b.partial_cmp(a).unwrap()); // highest bid first
        }

        for price in matched_prices {
            let queue = book.get_mut(&price).unwrap();
            while let Some(maker) = queue.front_mut() {
                let traded_qty = taker.qty.min(maker.qty);

                trades.push(MatchResult {
                    order_id: taker.order_id.clone(),
                    price,
                    qty: traded_qty,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    member_id: taker.member_id,
                    //TODO
                    order_status: 1,
                });

                taker.qty -= traded_qty;
                maker.qty -= traded_qty;

                if maker.qty == 0 {
                    queue.pop_front();
                }

                if taker.qty == 0 {
                    break;
                }
            }

            if queue.is_empty() {
                book.remove(&price);
            }

            if taker.qty == 0 {
                break;
            }
        }

        if taker.qty > 0 {
            self.insert(taker);
        }

        trades
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::types::*;

    fn make_order(order_id: &str, side: OrderSide, price: u64, qty: u64) -> Order {
        Order {
            order_id: order_id.to_string(),
            side: side,
            price: price,
            qty: qty,
            security_id: "000001".to_string(),
            member_id: 9527,
            uid: 9527,
            traded_qty: 0,
            timestamp: Utc::now().timestamp_millis(),
        }
    }

    #[test]
    fn test_exact_match() {
        let mut book = OrderBook::new();

        let maker = make_order("1", OrderSide::Sell, 100, 10);
        book.insert(maker);

        let taker = make_order("2", OrderSide::Buy, 100, 10);
        let trades = book.match_order(taker);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].qty, 10);
        assert!(book.asks.is_empty());
    }

    #[test]
    fn test_partial_match() {
        let mut book = OrderBook::new();

        let maker = make_order("1", OrderSide::Sell, 100, 5);
        book.insert(maker);

        let taker = make_order("2", OrderSide::Buy, 100, 10);
        let trades = book.match_order(taker);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].qty, 5);
        assert_eq!(book.bids.len(), 1);
    }

    #[test]
    fn test_no_match() {
        let mut book = OrderBook::new();

        let maker = make_order("1", OrderSide::Sell, 101, 5);
        book.insert(maker);

        let taker = make_order("2", OrderSide::Buy, 100, 5);
        let trades = book.match_order(taker);

        assert!(trades.is_empty());
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.asks.len(), 1);
    }
}
