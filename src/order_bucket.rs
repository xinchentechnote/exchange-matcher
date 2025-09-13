use indexmap::IndexMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{MatchEvent, Order, OrderStatus, RbCmd};
static TID_GEN: AtomicI64 = AtomicI64::new(1);
pub trait OrderBucket {
    fn put(&mut self, order: Order);
    fn remove(&mut self, oid: i64) -> Option<Order>;
    fn match_orders<F>(
        &mut self,
        volume_left: i64,
        trigger_cmd: &mut RbCmd,
        remove_order_callback: F,
    ) -> i64
    where
        F: FnMut(&Order);
    fn price(&self) -> i64;
    fn total_volume(&self) -> i64;
}

#[derive(Debug, Default)]
pub struct OrderBucketImpl {
    // 价格：每个价格一个 Bucket
    price: i64,
    // 总未成交量
    total_volume: i64,
    // insertion-ordered map: key=oid, value=Order
    entries: IndexMap<i64, Order>,
}

impl OrderBucketImpl {
    pub fn new(price: i64) -> Self {
        Self {
            price,
            total_volume: 0,
            entries: IndexMap::new(),
        }
    }

    fn gen_match_event(
        order: &Order,
        cmd: &mut RbCmd,
        full_match: bool,
        cmd_full_match: bool,
        traded: i64,
    ) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let tid = TID_GEN.fetch_add(1, Ordering::Relaxed);

        // current order match event (bidEvent)
        let bid_event = MatchEvent {
            session_id: order.session_id,
            timestamp: now_ms,
            mid: cmd.mid,
            oid: cmd.oid,
            status: if cmd_full_match {
                OrderStatus::TradeEd
            } else {
                OrderStatus::PartTrade
            },
            tid,
            volume: traded,
            price: order.price,
        };
        cmd.match_event_list.push(bid_event);

        // order match event (ofrEvent)
        let ofr_event = MatchEvent {
            session_id: cmd.session_id,
            timestamp: now_ms,
            mid: order.mid,
            oid: order.oid,
            status: if full_match {
                OrderStatus::TradeEd
            } else {
                OrderStatus::PartTrade
            },
            tid,
            volume: order.volume,
            price: order.price,
        };
        cmd.match_event_list.push(ofr_event);
    }
}

impl OrderBucket for OrderBucketImpl {
    fn put(&mut self, order: Order) {
        self.total_volume += order.volume - order.tvolume;
        self.entries.insert(order.oid, order);
    }

    fn remove(&mut self, oid: i64) -> Option<Order> {
        if let Some(order) = self.entries.shift_remove(&oid) {
            self.total_volume -= order.volume - order.tvolume;
            Some(order)
        } else {
            None
        }
    }

    fn match_orders<F>(
        &mut self,
        mut volume_left: i64,
        trigger_cmd: &mut RbCmd,
        mut remove_order_callback: F,
    ) -> i64
    where
        F: FnMut(&Order),
    {
        let mut volume_match = 0_i64;

        // interate orders use index
        let mut idx = 0_usize;
        while idx < self.entries.len() && volume_left > 0 {
            // get oid
            let oid = {
                let (oid_ref, _) = self.entries.get_index(idx).expect("index valid");
                *oid_ref
            };

            // current order
            {
                let order = self.entries.get_index_mut(idx).expect("index valid").1;
                let can_trade = order.remaining().max(0);
                if can_trade <= 0 {
                    // no trade
                    idx += 1;
                    continue;
                }
                let traded = volume_left.min(can_trade);

                volume_match += traded;
                order.tvolume += traded;
                volume_left -= traded;
                self.total_volume -= traded;

                let full_match = order.volume == order.tvolume;
                let cmd_full_match = volume_left == 0;
                // gen match event
                OrderBucketImpl::gen_match_event(
                    order,
                    trigger_cmd,
                    full_match,
                    cmd_full_match,
                    traded,
                );
            }

            // remove order if full matched
            let full_matched_now = {
                let order = self
                    .entries
                    .get(&oid)
                    .expect("exists after above borrow ends");
                order.volume == order.tvolume
            };

            if full_matched_now {
                // callback
                if let Some(order_done) = self.entries.get(&oid).cloned() {
                    remove_order_callback(&order_done);
                }
                // delete order
                let _ = self.entries.shift_remove(&oid);
                // continue
            } else {
                idx += 1;
            }
        }

        volume_match
    }

    #[inline]
    fn price(&self) -> i64 {
        self.price
    }

    #[inline]
    fn total_volume(&self) -> i64 {
        self.total_volume
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::types::OrderSide;

    use super::*;

    #[test]
    fn test_bucket_basic() {
        let mut bucket = OrderBucketImpl::new(45);

        bucket.put(Order {
            session_id: 1,
            oid: 11,
            mid: 1,
            price: 45,
            volume: 20,
            tvolume: 0,
            uid: 1,
            security_id: "000001".to_string(),
            side: OrderSide::Buy,
            timestamp: Utc::now().timestamp(),
        });
        bucket.put(Order {
            session_id: 1,
            oid: 20,
            mid: 2,
            price: 45,
            volume: 10,
            tvolume: 0,
            uid: 1,
            security_id: "000001".to_string(),
            side: OrderSide::Buy,
            timestamp: Utc::now().timestamp(),
        });

        let mut cmd = RbCmd {
            session_id: 1,
            security_id: "000001".to_string(),
            mid: 999,
            oid: 1000,
            match_event_list: vec![],
            side: OrderSide::Sell,
            price: 45,
            volume: 25,
            uid: 1,
        };

        let removed: &mut Vec<i64> = &mut vec![];
        let total = bucket.match_orders(25, &mut cmd, |o| removed.push(o.oid));

        assert_eq!(total, 25);
        assert_eq!(removed, &vec![11]);
        assert_eq!(bucket.total_volume(), 5);
        assert!(cmd.match_event_list.len() >= 2);
    }
}
