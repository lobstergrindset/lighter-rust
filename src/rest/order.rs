use crate::error::Result;
use crate::models::order::*;
use crate::models::order_book::*;
use crate::rest::client::LighterRestClient;

const DEFAULT_ACCOUNT_INACTIVE_ORDERS_LIMIT: &str = "100";

impl LighterRestClient {
    pub async fn get_order_books(&self) -> Result<OrderBooks> {
        self.get("/api/v1/orderBooks").await
    }

    pub async fn get_order_book_details(&self, market_id: i64) -> Result<OrderBookDetails> {
        let market_id = market_id.to_string();
        self.get_with_query(
            "/api/v1/orderBookDetails",
            &[("market_id", market_id.as_str())],
        )
        .await
    }

    pub async fn get_order_book_orders(
        &self,
        market_id: i64,
        limit: u32,
    ) -> Result<OrderBookDepth> {
        let market_id = market_id.to_string();
        let limit = limit.to_string();
        self.get_with_query(
            "/api/v1/orderBookOrders",
            &[("market_id", market_id.as_str()), ("limit", limit.as_str())],
        )
        .await
    }

    pub async fn get_account_active_orders(
        &self,
        account_index: i64,
        market_id: i64,
        auth: &str,
    ) -> Result<Orders> {
        let account_index = account_index.to_string();
        let market_id = market_id.to_string();
        self.get_with_auth(
            "/api/v1/accountActiveOrders",
            &[
                ("account_index", account_index.as_str()),
                ("market_id", market_id.as_str()),
            ],
            auth,
        )
        .await
    }

    pub async fn get_account_inactive_orders(
        &self,
        account_index: i64,
        market_id: i64,
        auth: &str,
        cursor: Option<&str>,
    ) -> Result<Orders> {
        let account_index = account_index.to_string();
        let market_id = market_id.to_string();
        let mut query: Vec<(&str, &str)> = vec![
            ("account_index", account_index.as_str()),
            ("market_id", market_id.as_str()),
            ("limit", DEFAULT_ACCOUNT_INACTIVE_ORDERS_LIMIT),
        ];
        if let Some(c) = cursor {
            query.push(("cursor", c));
        }
        self.get_with_auth("/api/v1/accountInactiveOrders", &query, auth)
            .await
    }

    pub async fn get_recent_trades(
        &self,
        market_id: i64,
        limit: u32,
    ) -> Result<crate::models::trade::Trades> {
        let market_id = market_id.to_string();
        let limit = limit.to_string();
        self.get_with_query(
            "/api/v1/recentTrades",
            &[("market_id", market_id.as_str()), ("limit", limit.as_str())],
        )
        .await
    }

    pub async fn get_trades(
        &self,
        market_id: i64,
        cursor: Option<&str>,
    ) -> Result<crate::models::trade::Trades> {
        let market_id = market_id.to_string();
        let mut query: Vec<(&str, &str)> = vec![
            ("market_id", market_id.as_str()),
            ("sort_by", "timestamp"),
            ("sort_dir", "desc"),
            ("limit", "100"),
        ];
        if let Some(c) = cursor {
            query.push(("cursor", c));
        }
        self.get_with_query("/api/v1/trades", &query).await
    }

    /// Fetch account-specific trades (includes PnL fields).
    pub async fn get_account_trades(
        &self,
        account_index: i64,
        auth: &str,
        cursor: Option<&str>,
    ) -> Result<crate::models::trade::Trades> {
        let account_index = account_index.to_string();
        let mut query: Vec<(&str, &str)> = vec![
            ("account_index", account_index.as_str()),
            ("sort_by", "timestamp"),
            ("sort_dir", "desc"),
            ("limit", "100"),
        ];
        if let Some(c) = cursor {
            query.push(("cursor", c));
        }
        self.get_with_auth("/api/v1/trades", &query, auth).await
    }
}
