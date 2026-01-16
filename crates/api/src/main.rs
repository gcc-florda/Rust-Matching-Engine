use actors::{AuditActor, AuditMsg, EngineMsg, GatewayActor, GatewayMsg, MatchingEngineActor};
use common::{NewOrder, Side};
use tokio::join;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let (gateway_tx, gateway_rx) = mpsc::channel::<GatewayMsg>(32);
    let (engine_tx, engine_rx) = mpsc::channel::<EngineMsg>(32);
    let (audit_tx, audit_rx) = mpsc::channel::<AuditMsg>(32);

    let audit_tx2 = audit_tx.clone();

    let gateway_actor: GatewayActor = GatewayActor::new(gateway_rx, engine_tx, audit_tx);
    let engine_actor: MatchingEngineActor = MatchingEngineActor::new(engine_rx, audit_tx2);
    let audit_actor = AuditActor::new(audit_rx);

    let gateway_handler = tokio::spawn(async move {
        gateway_actor.run().await;
    });

    let engine_handler = tokio::spawn(async move {
        engine_actor.run().await;
    });

    let audit_handler = tokio::spawn(async move {
        audit_actor.run().await;
    });

    let test_order: NewOrder = NewOrder {
        user_id: 1,
        side: Side::Buy,
        qty: 2,
        price: 100,
    };

    if gateway_tx
        .send(GatewayMsg::NewOrder(test_order))
        .await
        .is_err()
    {
        println!("ERROR SENDING NEWORDER");
    }

    if gateway_tx.send(GatewayMsg::Shutdown).await.is_err() {
        println!("ERROR SENDING SHUTDOWN")
    }

    let (_result_gateway, _result_engine, _result_audit) =
        join!(gateway_handler, engine_handler, audit_handler);

    Ok(())
}
