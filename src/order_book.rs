use std::cmp::Ordering as CmpOrdering;
use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;

use crate::order_bucket::{OrderBucket, OrderBucketImpl};
use crate::types::{CmdResultCode, L1MarketData, MatchEvent, Order, OrderSide, OrderStatus, RbCmd};

#[derive(Debug)]
pub struct OrderBook {
    security_id: String,
    // sell orders
    sell_buckets: BTreeMap<i64, OrderBucketImpl>,
    // buy orders
    buy_buckets: BTreeMap<RevPrice, OrderBucketImpl>,
    // order cache
    order_map: HashMap<i64, Order>,
}

// reverse price
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RevPrice(i64);

impl PartialOrd for RevPrice {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(other.0.cmp(&self.0)) // reverse
    }
}

impl Ord for RevPrice {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        other.0.cmp(&self.0)
    }
}

impl OrderBook {
    pub fn new(security_id: String) -> Self {
        Self {
            security_id,
            sell_buckets: BTreeMap::new(),
            buy_buckets: BTreeMap::new(),
            order_map: HashMap::new(),
        }
    }

    pub fn new_order(&mut self, cmd: &mut RbCmd) -> CmdResultCode {
        if self.order_map.contains_key(&cmd.oid) {
            return CmdResultCode::DuplicateOrderId;
        }

        let mut t_volume = 0;
        if cmd.side == OrderSide::Sell {
            let mut sub_buckets: Vec<*mut OrderBucketImpl> = self
                .buy_buckets
                .range(..=RevPrice(cmd.price))
                .map(|(_, b)| b as *const _ as *mut _)
                .collect();
            for b_ptr in sub_buckets {
                let bucket = unsafe { &mut *b_ptr };
                t_volume += bucket.match_orders(cmd.volume - t_volume, cmd, |order| {
                    self.order_map.remove(&order.oid);
                });
                if bucket.total_volume() == 0 {
                    let price = bucket.price();
                    self.buy_buckets.remove(&RevPrice(price));
                }
                if t_volume == cmd.volume {
                    break;
                }
            }
        } else {
            let mut sub_buckets: Vec<*mut OrderBucketImpl> = self
                .sell_buckets
                .range(..=cmd.price)
                .map(|(_, b)| b as *const _ as *mut _)
                .collect();
            for b_ptr in sub_buckets {
                let bucket = unsafe { &mut *b_ptr };
                t_volume += bucket.match_orders(cmd.volume - t_volume, cmd, |order| {
                    self.order_map.remove(&order.oid);
                });
                if bucket.total_volume() == 0 {
                    let price = bucket.price();
                    self.sell_buckets.remove(&price);
                }
                if t_volume == cmd.volume {
                    break;
                }
            }
        }

        if t_volume == cmd.volume {
            //全部成交
            return CmdResultCode::Success;
        }

        let order = Order {
            mid: cmd.mid,
            uid: cmd.uid,
            security_id: self.security_id.clone(),
            side: cmd.side,
            price: cmd.price,
            volume: cmd.volume,
            tvolume: t_volume,
            oid: cmd.oid,
            timestamp: Utc::now().timestamp_millis(),
        };

        if t_volume == 0 {
            //委托确认
            self.gen_match_event(cmd, OrderStatus::OrderEd);
        }
        //增加到订单簿
        if cmd.side == OrderSide::Sell {
            let bucket = self
                .sell_buckets
                .entry(cmd.price)
                .or_insert_with(|| OrderBucketImpl::new(cmd.price));
            bucket.put(order.clone());
        } else {
            let bucket = self
                .buy_buckets
                .entry(RevPrice(cmd.price))
                .or_insert_with(|| OrderBucketImpl::new(cmd.price));
            bucket.put(order.clone());
        }

        self.order_map.insert(order.oid, order);
        CmdResultCode::Success
    }

    fn gen_match_event(&self, cmd: &mut RbCmd, status: OrderStatus) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let mut ev = MatchEvent::default();
        ev.timestamp = now;
        ev.mid = cmd.mid;
        ev.oid = cmd.oid;
        ev.status = status;
        ev.volume = 0;
        cmd.match_event_list.push(ev);
    }

    pub fn cancel_order(&mut self, cmd: &mut RbCmd) -> CmdResultCode {
        let order = match self.order_map.get(&cmd.oid).cloned() {
            Some(o) => o,
            None => return CmdResultCode::InvalidOrderId,
        };

        let target_bucket = if order.side == OrderSide::Sell {
            self.sell_buckets.get_mut(&order.price)
        } else {
            self.buy_buckets.get_mut(&RevPrice(order.price))
        };

        if let Some(bucket) = target_bucket {
            bucket.remove(order.oid);
            if bucket.total_volume() == 0 {
                if order.side == OrderSide::Sell {
                    self.sell_buckets.remove(&order.price);
                } else {
                    self.buy_buckets.remove(&RevPrice(order.price));
                }
            }
        }

        // cancel event
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let mut ev = MatchEvent::default();
        ev.timestamp = now;
        ev.mid = order.mid;
        ev.oid = order.oid;
        ev.status = if order.tvolume == 0 {
            OrderStatus::CancelEd
        } else {
            OrderStatus::PartCancel
        };
        ev.volume = order.tvolume - order.volume;
        cmd.match_event_list.push(ev);

        CmdResultCode::Success
    }

    pub fn fill_code(&self, data: &mut L1MarketData) {
        data.security_id = self.security_id.clone();
    }

    pub fn fill_sells(&self, size: usize, data: &mut L1MarketData) {
        if size == 0 {
            data.sell_size = 0;
            return;
        }
        let mut i = 0;
        for bucket in self.sell_buckets.values() {
            data.sell_prices[i] = bucket.price();
            data.sell_volumes[i] = bucket.total_volume();
            i += 1;
            if i == size {
                break;
            }
        }
        data.sell_size = i;
    }

    pub fn fill_buys(&self, size: usize, data: &mut L1MarketData) {
        if size == 0 {
            data.buy_size = 0;
            return;
        }
        let mut i = 0;
        for bucket in self.buy_buckets.values() {
            data.buy_prices[i] = bucket.price();
            data.buy_volumes[i] = bucket.total_volume();
            i += 1;
            if i == size {
                break;
            }
        }
        data.buy_size = i;
    }

    pub fn limit_buy_bucket_size(&self, max_size: usize) -> usize {
        max_size.min(self.buy_buckets.len())
    }

    pub fn limit_sell_bucket_size(&self, max_size: usize) -> usize {
        max_size.min(self.sell_buckets.len())
    }
}
