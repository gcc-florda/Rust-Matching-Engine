#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct NewOrder {
    pub user_id: i32,
    pub side: Side,
    pub qty: i32,
    pub price: i32,
}

#[derive(Debug)]
pub struct ValidatedOrder {
    pub order_id: i32,
    pub user_id: i32,
    pub side: Side,
    pub qty: i32,
    pub price: i32,
}

#[derive(Debug)]
pub struct Trade {
    pub buy_order_id: i32,
    pub sell_order_id: i32,
    pub qty: i32,
    pub price: i32,
}

impl Trade {
    pub fn is_valid(&self) -> bool {
        self.price > 0 && self.qty > 0
    }
}
