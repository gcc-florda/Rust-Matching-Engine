use crate::messages::{AuditMsg, EngineMsg, GatewayMsg};
use common::{NewOrder, ValidatedOrder};
use tokio::sync::mpsc;

pub struct GatewayActor {
    next_id: i32,
    rx: mpsc::Receiver<GatewayMsg>,
    engine_tx: mpsc::Sender<EngineMsg>,
    audit_tx: mpsc::Sender<AuditMsg>,
}

impl GatewayActor {
    // si dejaba mut self no funcionaba
    async fn validate_new_order(&mut self, new_order: NewOrder) {
        if new_order.qty > 0 && new_order.price > 0 {
            self.next_id += 1;
            let validated_order_id: i32 = self.next_id;
            let valid_order: ValidatedOrder = ValidatedOrder {
                order_id: validated_order_id,
                user_id: new_order.user_id,
                side: new_order.side,
                qty: new_order.qty,
                price: new_order.price,
            };
            if self
                .engine_tx
                .send(EngineMsg::Order(valid_order))
                .await
                .is_err()
            {
                println!("ERROR SENDING VALID ORDER");
            }
        } else {
            if self.audit_tx.send(AuditMsg::RejectedOrder).await.is_err() {
                println!("ERROR SENDING REJECTED ORDER");
            }
        }
    }

    // por que tiene que ser mut self ?
    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                GatewayMsg::NewOrder(new_order) => {
                    println!("New order received: {:?}", new_order);
                    self.validate_new_order(new_order).await;
                }
                GatewayMsg::Shutdown => {
                    if self.engine_tx.send(EngineMsg::Shutdown).await.is_err() {
                        println!("ERROR SENDING SHUTDOWN");
                    };
                    break;
                }
            }
        }
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
            audit_tx,
        }
    }
}
