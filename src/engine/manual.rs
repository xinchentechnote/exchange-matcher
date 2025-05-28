pub enum ManualActionType {
    MatchWith(OrderRequest), // 手动构造对手方订单
    Reject(String),          // 拒单理由
    ForceFill(f64, u64),     // 强制成交（价格/数量）
}

pub struct ManualControl;

impl ManualControl {
    pub fn perform_action(&mut self, order_id: &str, action: ManualActionType);
}
