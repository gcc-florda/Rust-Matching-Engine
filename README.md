# Order Processing System (3-Actor Model)

This project consists of implementing a simplified trading system using the Actor Model in Rust, leveraging tokio for asynchronous task management and mpsc (multi-producer, single-consumer) channels for inter-actor communication.

For conceptual reference on how matching engines work, see: [Understanding Matching Engines (Binance Academy)](https://academy.binance.com/en/articles/understanding-matching-engines).

## Architecture

The system is composed of three main actors, each running in its own asynchronous task with its own state and message loop.

### 1. Actor A: Gateway

**Responsibility:** Receives, validates, and normalizes incoming orders.

- **State:** A `next_id` counter to assign unique identifiers to orders.

- **Messages:**
  - `NewOrder { user_id, side, qty, price }`
  - `Shutdown`

- **Logic:**
  - **Upon NewOrder:** Validate that `qty > 0` and `price > 0`. If valid, assign `order_id = next_id`, increment the counter, and forward a `ValidatedOrder` to Actor B.
  - **Upon Shutdown:** Forward the `Shutdown` signal to Actor B and terminate the loop.

### 2. Actor B: Matching Engine

**Responsibility:** Manages a simplified order book and generates trades.

- **State:**
  - `open_orders`: A collection (e.g., `Vec` or `BTreeMap`) of buy and sell orders.

- **Messages:**
  - `ValidatedOrder { ... }`
  - `Cancel { order_id }` (Optional)
  - `Shutdown`

- **Logic:**
  - Check if the incoming order "crosses" with existing ones (e.g., `buy.price >= sell.price`).
  - If a match occurs: Generate a `Trade { buy_id, sell_id, qty, price }` and send it to Actor C.
  - If no match: Store the order in `open_orders`.
  - **Upon Shutdown:** Forward the `Shutdown` signal to Actor C and terminate.

### 3. Actor C: Logger / Audit

**Responsibility:** Records system activity and generates final statistics.

- **State:**
  - `trades`: A history of all executed trades.
  - `total_volume`: Sum of all traded quantities.
  - `price_history`: Tracking of execution prices.

- **Messages:**
  - `Trade { ... }`
  - `RejectedOrder { reason, ... }` (Optional)
  - `Shutdown`

- **Logic:**
  - **Upon Trade:** Update volume and price statistics.
  - **Upon Shutdown:** Print a final summary report (total trades, total volume, average price, etc.) and terminate.

## Requirements & Steps

1. **Message Definition:** Define an enum `Msg` (or specific enums for each actor: `GatewayMsg`, `EngineMsg`, `AuditMsg`).

2. **Channel Setup:** Create the following communication pipes:
   - `main -> Gateway`
   - `Gateway -> Matching Engine`
   - `Matching Engine -> Audit`

3. **Task Spawning:** Implement each actor using `tokio::spawn(async move { ... })` with a loop and a `match rx.recv().await` block.

4. **Simulation:** In the main function, inject a test sequence:
   - 3 Buy orders and 3 Sell orders designed to trigger matches.
   - 1 Invalid order (e.g., `qty: 0`) to test Gateway rejection.

5. **Graceful Shutdown:** Send a `Shutdown` signal and verify:
   - The system does not hang.
   - Actors terminate in the correct order.
   - The Audit actor displays the final summary.

## Actor Model Constraints

To ensure an authentic Actor Model implementation:

- **No Shared State:** Actors must not expose their internal state. All interaction must happen via messages.

- **Bonus (State Querying):** If you need to "query" an actor's state from main, implement a `GetStats { reply_to: oneshot::Sender<Stats> }` pattern.

## Tech Stack

- **Rust**
- **Tokio** (Async runtime)
- `std::sync::mpsc` or `tokio::sync::mpsc`
