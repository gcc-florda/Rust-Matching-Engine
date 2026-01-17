use crate::messages::AuditMsg;
use common::{AuditError, Trade};
use tokio::sync::mpsc;

pub struct AuditActor {
    trades: Vec<Trade>,
    volume_total: i32,
    rejected_orders: i32,
    last_prices: Vec<i32>,
    rx: mpsc::Receiver<AuditMsg>,
}

impl AuditActor {
    // pierdo ownership de self sin en & --> pierdo el actor y ya no lo podria seguir usando despues
    // &mut self --> muto estado interno
    fn stats(&mut self, trade: Trade) -> Result<(), AuditError> {
        if trade.is_valid() {
            println!(
                "Processing Buy Order: {} Sell Order: {} ",
                trade.buy_order_id, trade.sell_order_id
            );
            self.last_prices.push(trade.price);
            self.volume_total += trade.qty;
            self.trades.push(trade);
        } else {
            Err(AuditError::InvalidTrade)?;
        }
        Ok(())
    }

    fn rejected_order(&mut self) {
        self.rejected_orders += 1;
    }

    pub async fn run(mut self) -> Result<(), AuditError> {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                AuditMsg::Trade(trade) => {
                    println!("TRADE RECEIVED: {:?}", trade);
                    self.stats(trade)?;
                }
                AuditMsg::RejectedOrder => self.rejected_order(),
                AuditMsg::Shutdown => {
                    println!("Total Volume: {}", self.volume_total);
                    println!("Number of Trades: {}", self.trades.len());
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn new(rx: mpsc::Receiver<AuditMsg>) -> Self {
        Self {
            trades: vec![],
            volume_total: 0,
            rejected_orders: 0,
            last_prices: vec![],
            rx,
        }
    }
}
