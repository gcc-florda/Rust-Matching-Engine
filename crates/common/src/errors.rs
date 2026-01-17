use tokio::task::JoinError;

#[derive(Debug)]
pub enum GatewayError {
    InvalidQty { qty: i32 },
    InvalidPrice { price: i32 },
    GatewayChannelClosed,
    EngineChannelClosed,
}

#[derive(Debug)]
pub enum EngineError {
    AuditChannelClosed,
}

#[derive(Debug)]
pub enum AuditError {
    InvalidTrade,
}

#[derive(Debug)]
pub enum MainError {
    Gateway(GatewayError),
    Engine(EngineError),
    Audit(AuditError),
    GatewayChannelClosed,
    TaskJoin(JoinError),
}

impl From<GatewayError> for MainError {
    fn from(e: GatewayError) -> Self {
        MainError::Gateway(e)
    }
}
impl From<EngineError> for MainError {
    fn from(e: EngineError) -> Self {
        MainError::Engine(e)
    }
}
impl From<AuditError> for MainError {
    fn from(e: AuditError) -> Self {
        MainError::Audit(e)
    }
}
impl From<JoinError> for MainError {
    fn from(e: JoinError) -> Self {
        MainError::TaskJoin(e)
    }
}
