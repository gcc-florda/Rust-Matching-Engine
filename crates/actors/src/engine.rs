use crate::messages::{AuditMsg, EngineMsg};
use common::{EngineError, Side, Trade, ValidatedOrder};
use std::cmp;
use tokio::sync::mpsc;

pub struct MatchingEngineActor {
    open_orders: Vec<ValidatedOrder>,
    rx: mpsc::Receiver<EngineMsg>,
    audit_tx: mpsc::Sender<AuditMsg>,
}

impl MatchingEngineActor {
    async fn process_order(&mut self, order: ValidatedOrder) -> Result<(), EngineError> {
        let mut found = false;
        let mut new_order: Option<ValidatedOrder> = None;
        let mut remove_idx: Option<usize> = None;
        for (i, open_order) in self.open_orders.iter_mut().enumerate() {
            // BUY cruza con SELL si sell.price <= buy.price
            // a mejorar --> ahora estoy matcheando contra la primera orden que cruza
            // Para un BUY deberías elegir la SELL con menor precio (best ask)
            // Para un SELL deberías elegir la BUY con mayor precio (best bid)
            if order.side == Side::Buy
                && open_order.side == Side::Sell
                && open_order.price <= order.price
            {
                let trade_qty = cmp::min(order.qty, open_order.qty);
                let trade = Trade {
                    buy_order_id: order.order_id,
                    sell_order_id: open_order.order_id,
                    qty: trade_qty,
                    price: open_order.price,
                };

                if open_order.qty > order.qty {
                    // queda remanente del open_order (se reduce)
                    open_order.qty = open_order.qty - trade_qty;
                } else if order.qty > open_order.qty {
                    // open_order se llena -> se elimina; queda remanente del order
                    new_order = Some(ValidatedOrder {
                        order_id: order.order_id,
                        user_id: order.user_id,
                        side: order.side,
                        qty: order.qty - trade_qty,
                        price: order.price,
                    });
                    remove_idx = Some(i);
                } else if order.qty == open_order.qty {
                    // iguales -> open_order se elimina
                    remove_idx = Some(i);
                }

                self.audit_tx
                    .send(AuditMsg::Trade(trade))
                    .await
                    .map_err(|_| EngineError::AuditChannelClosed)?;

                found = true;

                break;
            // SELL cruza con BUY si buy.price >= sell.price
            } else if order.side == Side::Sell
                && open_order.side == Side::Buy
                && open_order.price >= order.price
            {
                let trade_qty = cmp::min(order.qty, open_order.qty);
                let trade = Trade {
                    buy_order_id: open_order.order_id,
                    sell_order_id: order.order_id,
                    qty: trade_qty,
                    price: open_order.price,
                };

                if open_order.qty > order.qty {
                    open_order.qty = open_order.qty - trade_qty;
                } else if order.qty > open_order.qty {
                    new_order = Some(ValidatedOrder {
                        order_id: order.order_id,
                        user_id: order.user_id,
                        side: order.side,
                        qty: order.qty - trade_qty,
                        price: order.price,
                    });
                    remove_idx = Some(i);
                } else if order.qty == open_order.qty {
                    remove_idx = Some(i);
                }

                self.audit_tx
                    .send(AuditMsg::Trade(trade))
                    .await
                    .map_err(|_| EngineError::AuditChannelClosed)?;

                found = true;

                break;
            }
        }

        if let Some(idx) = remove_idx {
            self.open_orders.remove(idx);
        }
        if let Some(rem) = new_order {
            self.open_orders.push(rem);
        }

        if !found {
            self.open_orders.push(order);
        }

        println!("[ENGINE] OPEN ORDERS: {:?}", self.open_orders);
        Ok(())
    }

    pub async fn run(mut self) -> Result<(), EngineError> {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                EngineMsg::Order(order) => {
                    println!("[ENGINE] VALID ORDER RECEIVED: {:?}", order);
                    self.process_order(order).await?
                }
                EngineMsg::Shutdown => {
                    self.audit_tx
                        .send(AuditMsg::Shutdown)
                        .await
                        .map_err(|_| EngineError::AuditChannelClosed)?;
                    break
                }
            }
        }

        Ok(())
    }

    pub fn new(rx: mpsc::Receiver<EngineMsg>, audit_tx: mpsc::Sender<AuditMsg>) -> Self {
        Self {
            open_orders: vec![],
            rx,
            audit_tx,
        }
    }
}
