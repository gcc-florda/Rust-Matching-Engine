use actors::{AuditActor, AuditMsg, EngineMsg, GatewayActor, GatewayMsg, MatchingEngineActor};
use common::{MainError, NewOrder, Side};
use tokio::join;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let (gateway_tx, gateway_rx) = mpsc::channel::<GatewayMsg>(32);
    let (engine_tx, engine_rx) = mpsc::channel::<EngineMsg>(32);
    let (audit_tx, audit_rx) = mpsc::channel::<AuditMsg>(32);

    let audit_tx2 = audit_tx.clone();

    let gateway_actor: GatewayActor = GatewayActor::new(gateway_rx, engine_tx, audit_tx);
    let engine_actor: MatchingEngineActor = MatchingEngineActor::new(engine_rx, audit_tx2);
    let audit_actor = AuditActor::new(audit_rx);

    let gateway_handler =
        tokio::spawn(async move { gateway_actor.run().await.map_err(MainError::from) });

    let engine_handler =
        tokio::spawn(async move { engine_actor.run().await.map_err(MainError::from) });

    let audit_handler =
        tokio::spawn(async move { audit_actor.run().await.map_err(MainError::from) });

    let test_order_1: NewOrder = NewOrder {
        user_id: 1,
        side: Side::Sell,
        qty: 2,
        price: 100,
    };

    let test_order_2: NewOrder = NewOrder {
        user_id: 2,
        side: Side::Buy,
        qty: 1,
        price: 50,
    };

    let test_order_3: NewOrder = NewOrder {
        user_id: 3,
        side: Side::Buy,
        qty: 1,
        price: 200,
    };

    let orders = vec![test_order_1, test_order_2, test_order_3];

    for order in orders {
        gateway_tx
        .send(GatewayMsg::NewOrder(order))
        .await
        .map_err(|_| MainError::GatewayChannelClosed)?
    }

    gateway_tx
        .send(GatewayMsg::Shutdown)
        .await
        .map_err(|_| MainError::GatewayChannelClosed)?;

    let (result_gateway, result_engine, result_audit) =
        join!(gateway_handler, engine_handler, audit_handler);

    result_gateway??;
    result_engine??;
    result_audit??;

    Ok(())
}
