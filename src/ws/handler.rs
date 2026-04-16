use std::collections::HashMap;

use crate::models::order::Order;
use crate::models::order_book::PriceLevel;
use crate::models::ws::{
    WsAccountAllAssetsUpdate, WsAccountAllPositionsUpdate, WsAccountAllTradesUpdate,
    WsAccountAllUpdate, WsAccountSpotAvgEntryPricesUpdate, WsMarketStatsUpdate, WsOrderBook,
    WsOrderBookState, WsUserStatsUpdate,
};

/// Manages local order book state from WebSocket delta updates.
#[derive(Debug, Default)]
pub struct OrderBookHandler {
    states: HashMap<i64, WsOrderBookState>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OrderBookUpdateError {
    #[error("market {market_id} received an order book delta before a snapshot")]
    MissingSnapshot { market_id: i64 },
    #[error(
        "market {market_id} order book nonce gap: expected begin_nonce {expected_begin_nonce}, got {actual_begin_nonce} (previous nonce {previous_nonce})"
    )]
    NonContiguousNonce {
        market_id: i64,
        expected_begin_nonce: i64,
        actual_begin_nonce: i64,
        previous_nonce: i64,
    },
}

impl OrderBookHandler {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Initialize or replace the full order book snapshot for a market.
    pub fn set_snapshot(&mut self, market_id: i64, state: WsOrderBookState) {
        self.states.insert(market_id, state);
    }

    /// Apply a delta update to the local order book state.
    /// Returns the updated state if it exists.
    ///
    /// After applying deltas, bids are sorted descending and asks ascending
    /// by price so that `.first()` always returns the best level.
    pub fn apply_update(
        &mut self,
        market_id: i64,
        update: &WsOrderBook,
    ) -> Result<&WsOrderBookState, OrderBookUpdateError> {
        let state = self
            .states
            .get_mut(&market_id)
            .ok_or(OrderBookUpdateError::MissingSnapshot { market_id })?;
        if update.begin_nonce != state.nonce {
            return Err(OrderBookUpdateError::NonContiguousNonce {
                market_id,
                expected_begin_nonce: state.nonce,
                actual_begin_nonce: update.begin_nonce,
                previous_nonce: state.nonce,
            });
        }
        apply_price_level_deltas(&mut state.asks, &update.asks);
        apply_price_level_deltas(&mut state.bids, &update.bids);
        sort_price_levels_ascending(&mut state.asks);
        sort_price_levels_descending(&mut state.bids);
        state.nonce = update.nonce;
        Ok(state)
    }

    /// Get the current order book state for a market.
    pub fn get(&self, market_id: i64) -> Option<&WsOrderBookState> {
        self.states.get(&market_id)
    }
}

/// Apply delta price levels to an existing price level list.
/// If a delta has size 0.0, the matching price level is removed.
/// If the price already exists, its size is updated.
/// If the price is new, the level is appended.
fn apply_price_level_deltas(existing: &mut Vec<PriceLevel>, deltas: &[PriceLevel]) {
    for delta in deltas {
        let delta_price = match delta.price_f64() {
            Some(p) => p,
            None => continue,
        };

        let is_removal = delta.size_f64().map(|v| v == 0.0).unwrap_or(false);

        if let Some(pos) = existing
            .iter()
            .position(|l| l.price_f64() == Some(delta_price))
        {
            if is_removal {
                existing.remove(pos);
            } else {
                existing[pos].size.clone_from(&delta.size);
            }
        } else if !is_removal {
            existing.push(delta.clone());
        }
    }
}

fn parse_price(level: &PriceLevel) -> f64 {
    level.price_f64().unwrap_or(0.0)
}

/// Sort asks ascending (best/lowest ask first).
fn sort_price_levels_ascending(levels: &mut [PriceLevel]) {
    levels.sort_by(|a, b| {
        parse_price(a)
            .partial_cmp(&parse_price(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Sort bids descending (best/highest bid first).
fn sort_price_levels_descending(levels: &mut [PriceLevel]) {
    levels.sort_by(|a, b| {
        parse_price(b)
            .partial_cmp(&parse_price(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Manages local account state from WebSocket updates.
#[derive(Debug, Default)]
pub struct AccountHandler {
    states: HashMap<i64, WsAccountAllUpdate>,
}

impl AccountHandler {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Set or replace the full account state.
    pub fn set_state(&mut self, account_id: i64, state: WsAccountAllUpdate) {
        self.states.insert(account_id, state);
    }

    /// Get the current account state.
    pub fn get(&self, account_id: i64) -> Option<&WsAccountAllUpdate> {
        self.states.get(&account_id)
    }
}

/// Manages local account orders state from WebSocket updates.
#[derive(Debug, Default)]
pub struct AccountOrdersHandler {
    /// Legacy/global view: market_id -> orders.
    states_by_market: HashMap<i64, Vec<Order>>,
    /// Precise view: (account_id, market_id) -> orders.
    states_by_account_market: HashMap<(i64, i64), Vec<Order>>,
}

impl AccountOrdersHandler {
    pub fn new() -> Self {
        Self {
            states_by_market: HashMap::new(),
            states_by_account_market: HashMap::new(),
        }
    }

    /// Set or replace the orders for a market.
    pub fn set_orders(&mut self, market_id: i64, orders: Vec<Order>) {
        self.states_by_market.insert(market_id, orders);
    }

    /// Set or replace the orders for an (account, market) pair.
    pub fn set_orders_for_account(&mut self, account_id: i64, market_id: i64, orders: Vec<Order>) {
        self.states_by_account_market
            .insert((account_id, market_id), orders);
    }

    /// Get the current orders for a market.
    pub fn get(&self, market_id: i64) -> Option<&Vec<Order>> {
        self.states_by_market.get(&market_id)
    }

    /// Get the current orders for an (account, market) pair.
    pub fn get_for_account(&self, account_id: i64, market_id: i64) -> Option<&Vec<Order>> {
        self.states_by_account_market.get(&(account_id, market_id))
    }
}

/// Manages local account-all-positions state from WebSocket updates.
#[derive(Debug, Default)]
pub struct AccountAllPositionsHandler {
    /// account_id -> latest account_all_positions payload
    states: HashMap<i64, WsAccountAllPositionsUpdate>,
}

impl AccountAllPositionsHandler {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Set or replace the latest account_all_positions payload.
    pub fn set_state(&mut self, account_id: i64, state: WsAccountAllPositionsUpdate) {
        self.states.insert(account_id, state);
    }

    /// Get the latest account_all_positions payload for an account.
    pub fn get(&self, account_id: i64) -> Option<&WsAccountAllPositionsUpdate> {
        self.states.get(&account_id)
    }
}

/// Manages local account-all-assets state from WebSocket updates.
#[derive(Debug, Default)]
pub struct AccountAllAssetsHandler {
    /// account_id -> latest account_all_assets payload
    states: HashMap<i64, WsAccountAllAssetsUpdate>,
}

impl AccountAllAssetsHandler {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Set or replace the latest account_all_assets payload.
    pub fn set_state(&mut self, account_id: i64, state: WsAccountAllAssetsUpdate) {
        self.states.insert(account_id, state);
    }

    /// Get the latest account_all_assets payload for an account.
    pub fn get(&self, account_id: i64) -> Option<&WsAccountAllAssetsUpdate> {
        self.states.get(&account_id)
    }
}

/// Manages local account-spot-avg-entry-prices state from WebSocket updates.
#[derive(Debug, Default)]
pub struct AccountSpotAvgEntryPricesHandler {
    /// account_id -> latest account_spot_avg_entry_prices payload
    states: HashMap<i64, WsAccountSpotAvgEntryPricesUpdate>,
}

impl AccountSpotAvgEntryPricesHandler {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Set or replace the latest account_spot_avg_entry_prices payload.
    pub fn set_state(&mut self, account_id: i64, state: WsAccountSpotAvgEntryPricesUpdate) {
        self.states.insert(account_id, state);
    }

    /// Get the latest account_spot_avg_entry_prices payload for an account.
    pub fn get(&self, account_id: i64) -> Option<&WsAccountSpotAvgEntryPricesUpdate> {
        self.states.get(&account_id)
    }
}

/// Manages local account-all-trades state from WebSocket updates.
#[derive(Debug, Default)]
pub struct AccountAllTradesHandler {
    /// account_id -> latest account_all_trades payload
    states: HashMap<i64, WsAccountAllTradesUpdate>,
}

impl AccountAllTradesHandler {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Set or replace the latest account_all_trades payload.
    pub fn set_state(&mut self, account_id: i64, state: WsAccountAllTradesUpdate) {
        self.states.insert(account_id, state);
    }

    /// Get the latest account_all_trades payload for an account.
    pub fn get(&self, account_id: i64) -> Option<&WsAccountAllTradesUpdate> {
        self.states.get(&account_id)
    }
}

/// Manages local user stats state from WebSocket updates.
#[derive(Debug, Default)]
pub struct UserStatsHandler {
    /// account_id -> latest stats
    states: HashMap<i64, WsUserStatsUpdate>,
}

impl UserStatsHandler {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Set or replace the user stats for an account.
    pub fn set_state(&mut self, account_id: i64, state: WsUserStatsUpdate) {
        self.states.insert(account_id, state);
    }

    /// Get the current user stats for an account.
    pub fn get(&self, account_id: i64) -> Option<&WsUserStatsUpdate> {
        self.states.get(&account_id)
    }
}

/// Manages local market stats state from WebSocket updates.
#[derive(Debug, Default)]
pub struct MarketStatsHandler {
    /// market_id -> latest market stats payload
    states: HashMap<i64, WsMarketStatsUpdate>,
}

impl MarketStatsHandler {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Set or replace the market stats for a market.
    pub fn set_state(&mut self, market_id: i64, state: WsMarketStatsUpdate) {
        self.states.insert(market_id, state);
    }

    /// Get the current market stats for a market.
    pub fn get(&self, market_id: i64) -> Option<&WsMarketStatsUpdate> {
        self.states.get(&market_id)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::models::order::Order;
    use crate::models::order_book::PriceLevel;
    use crate::models::ws::{
        WsAccountAllAssetsUpdate, WsAccountAllTradesUpdate, WsAccountSpotAvgEntryPricesUpdate,
        WsOrderBook, WsOrderBookState,
    };

    use super::{
        AccountAllAssetsHandler, AccountAllTradesHandler, AccountOrdersHandler,
        AccountSpotAvgEntryPricesHandler, OrderBookHandler, OrderBookUpdateError,
    };

    fn level(price: &str, size: &str) -> PriceLevel {
        PriceLevel {
            price: price.to_string(),
            size: size.to_string(),
        }
    }

    fn snapshot_state() -> WsOrderBookState {
        WsOrderBookState {
            asks: vec![level("101", "2"), level("102", "3")],
            bids: vec![level("99", "4"), level("98", "5")],
            nonce: 10,
        }
    }

    fn update_book(
        begin_nonce: i64,
        nonce: i64,
        asks: Vec<PriceLevel>,
        bids: Vec<PriceLevel>,
    ) -> WsOrderBook {
        WsOrderBook {
            code: 0,
            asks,
            bids,
            offset: 0,
            nonce,
            begin_nonce,
        }
    }

    fn sample_order(order_index: i64, market_index: i64) -> Order {
        Order {
            order_index,
            client_order_index: 0,
            order_id: String::new(),
            client_order_id: String::new(),
            market_index,
            owner_account_index: 0,
            initial_base_amount: "1".to_string(),
            price: "1".to_string(),
            nonce: 0,
            remaining_base_amount: "1".to_string(),
            is_ask: true,
            base_size: 0,
            base_price: 0,
            filled_base_amount: "0".to_string(),
            filled_quote_amount: String::new(),
            side: "ask".to_string(),
            order_type: "limit".to_string(),
            time_in_force: "good-till-time".to_string(),
            reduce_only: false,
            trigger_price: String::new(),
            order_expiry: 0,
            status: "open".to_string(),
            trigger_status: String::new(),
            trigger_time: 0,
            parent_order_index: 0,
            parent_order_id: String::new(),
            to_trigger_order_id_0: String::new(),
            to_trigger_order_id_1: String::new(),
            to_cancel_order_id_0: String::new(),
            block_height: 0,
            timestamp: 0,
            created_at: 0,
            updated_at: 0,
            transaction_time: 0,
        }
    }

    #[test]
    fn account_scoped_orders_do_not_clobber_each_other() {
        let mut handler = AccountOrdersHandler::new();
        handler.set_orders_for_account(100, 89, vec![sample_order(1, 89)]);
        handler.set_orders_for_account(200, 89, vec![sample_order(2, 89)]);

        let a = handler.get_for_account(100, 89).unwrap();
        let b = handler.get_for_account(200, 89).unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(b.len(), 1);
        assert_eq!(a[0].order_index, 1);
        assert_eq!(b[0].order_index, 2);
    }

    #[test]
    fn account_all_assets_state_is_stored_by_account() {
        let mut handler = AccountAllAssetsHandler::new();
        handler.set_state(
            1234,
            WsAccountAllAssetsUpdate {
                msg_type: "update/account_all_assets".to_string(),
                channel: "account_all_assets:1234".to_string(),
                assets: vec![],
            },
        );

        assert!(handler.get(1234).is_some());
        assert!(handler.get(9999).is_none());
    }

    #[test]
    fn account_all_trades_state_is_stored_by_account() {
        let mut handler = AccountAllTradesHandler::new();
        handler.set_state(
            1234,
            WsAccountAllTradesUpdate {
                msg_type: "update/account_all_trades".to_string(),
                channel: "account_all_trades:1234".to_string(),
                trades: HashMap::new(),
                total_volume: 0.0,
                monthly_volume: 0.0,
                weekly_volume: 0.0,
                daily_volume: 0.0,
            },
        );

        assert!(handler.get(1234).is_some());
        assert!(handler.get(9999).is_none());
    }

    #[test]
    fn account_spot_avg_entry_prices_state_is_stored_by_account() {
        let mut handler = AccountSpotAvgEntryPricesHandler::new();
        handler.set_state(
            1234,
            WsAccountSpotAvgEntryPricesUpdate {
                msg_type: "update/account_spot_avg_entry_prices".to_string(),
                channel: "account_spot_avg_entry_prices/1234".to_string(),
                avg_entry_prices: HashMap::new(),
            },
        );

        assert!(handler.get(1234).is_some());
        assert!(handler.get(9999).is_none());
    }

    #[test]
    fn order_book_snapshot_initializes_state_and_nonce() {
        let mut handler = OrderBookHandler::new();
        handler.set_snapshot(42, snapshot_state());

        let state = handler.get(42).unwrap();
        assert_eq!(state.nonce, 10);
        assert_eq!(state.bids[0].price, "99");
        assert_eq!(state.asks[0].price, "101");
    }

    #[test]
    fn contiguous_order_book_delta_updates_state() {
        let mut handler = OrderBookHandler::new();
        handler.set_snapshot(42, snapshot_state());

        let updated = handler
            .apply_update(
                42,
                &update_book(
                    10,
                    11,
                    vec![level("101", "1"), level("103", "7")],
                    vec![level("99", "0"), level("100", "6")],
                ),
            )
            .unwrap();

        assert_eq!(updated.nonce, 11);
        assert_eq!(updated.asks.len(), 3);
        assert_eq!(updated.asks[0].price, "101");
        assert_eq!(updated.asks[0].size, "1");
        assert_eq!(updated.bids.len(), 2);
        assert_eq!(updated.bids[0].price, "100");
        assert_eq!(updated.bids[0].size, "6");
    }

    #[test]
    fn order_book_nonce_gap_is_rejected() {
        let mut handler = OrderBookHandler::new();
        handler.set_snapshot(42, snapshot_state());

        let err = handler
            .apply_update(42, &update_book(9, 11, vec![level("101", "1")], Vec::new()))
            .unwrap_err();

        assert_eq!(
            err,
            OrderBookUpdateError::NonContiguousNonce {
                market_id: 42,
                expected_begin_nonce: 10,
                actual_begin_nonce: 9,
                previous_nonce: 10,
            }
        );
    }
}
