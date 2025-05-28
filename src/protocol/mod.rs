pub trait ProtocolAdapter {
    fn parse(&self, raw: &[u8]) -> Result<OrderRequest, String>;
    fn format(&self, report: &TradeReport) -> Vec<u8>;
}
