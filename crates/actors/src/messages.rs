use common::{NewOrder, Trade, ValidatedOrder};

#[derive(Debug)]
pub enum GatewayMsg {
    NewOrder(NewOrder),
    Shutdown,
}

#[derive(Debug)]
pub enum EngineMsg {
    Order(ValidatedOrder),
    Shutdown,
}

#[derive(Debug)]
pub enum AuditMsg {
    Trade(Trade),
    RejectedOrder,
    Shutdown,
}
