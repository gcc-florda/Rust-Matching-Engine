use crate::messages::{AuditMsg, EngineMsg, GatewayMsg};
use common::{GatewayError, NewOrder, ValidatedOrder};
use tokio::sync::mpsc;

pub struct GatewayActor {
    next_id: i32,
    rx: mpsc::Receiver<GatewayMsg>,
    engine_tx: mpsc::Sender<EngineMsg>,
    _audit_tx: mpsc::Sender<AuditMsg>,
}

impl GatewayActor {
    // si dejaba mut self no funcionaba
    async fn validate_new_order(&mut self, new_order: NewOrder) -> Result<(), GatewayError> {
        let qty = new_order.qty;
        let price = new_order.price;

        if qty < 0 {
            Err(GatewayError::InvalidQty { qty })
        } else if price < 0 {
            Err(GatewayError::InvalidPrice { price })
        } else {
            self.next_id += 1;
            let order_id: i32 = self.next_id;
            let valid_order: ValidatedOrder = ValidatedOrder {
                order_id,
                user_id: new_order.user_id,
                side: new_order.side,
                qty,
                price,
            };
            self.engine_tx
                .send(EngineMsg::Order(valid_order))
                .await
                .map_err(|_| GatewayError::EngineChannelClosed)?;

            Ok(())
        }
    }

    // por que tiene que ser mut self ?
    pub async fn run(mut self) -> Result<(), GatewayError> {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                GatewayMsg::NewOrder(new_order) => self.validate_new_order(new_order).await?,
                GatewayMsg::Shutdown => {
                    self.engine_tx
                        .send(EngineMsg::Shutdown)
                        .await
                        .map_err(|_| GatewayError::EngineChannelClosed)?;
                }
            }
        }

        Ok(())
    }

    pub fn new(
        rx: mpsc::Receiver<GatewayMsg>,
        engine_tx: mpsc::Sender<EngineMsg>,
        audit_tx: mpsc::Sender<AuditMsg>,
    ) -> Self {
        Self {
            next_id: 0,
            rx,
            engine_tx,
            _audit_tx: audit_tx,
        }
    }
}
