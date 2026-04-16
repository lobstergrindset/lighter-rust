use crate::error::Result;
use crate::models::bridge::*;
use crate::rest::client::LighterRestClient;

impl LighterRestClient {
    pub async fn get_deposit_history(
        &self,
        account_index: i64,
        auth: &str,
        cursor: Option<&str>,
    ) -> Result<DepositHistory> {
        let account_index = account_index.to_string();
        let mut query: Vec<(&str, &str)> = vec![("account_index", account_index.as_str())];
        if let Some(c) = cursor {
            query.push(("cursor", c));
        }
        self.get_with_auth("/api/v1/deposit/history", &query, auth)
            .await
    }

    pub async fn get_withdraw_history(
        &self,
        account_index: i64,
        auth: &str,
        cursor: Option<&str>,
    ) -> Result<WithdrawHistory> {
        let account_index = account_index.to_string();
        let mut query: Vec<(&str, &str)> = vec![("account_index", account_index.as_str())];
        if let Some(c) = cursor {
            query.push(("cursor", c));
        }
        self.get_with_auth("/api/v1/withdraw/history", &query, auth)
            .await
    }

    pub async fn get_transfer_history(
        &self,
        account_index: i64,
        auth: &str,
        cursor: Option<&str>,
    ) -> Result<TransferHistory> {
        let account_index = account_index.to_string();
        let mut query: Vec<(&str, &str)> = vec![("account_index", account_index.as_str())];
        if let Some(c) = cursor {
            query.push(("cursor", c));
        }
        self.get_with_auth("/api/v1/transfer/history", &query, auth)
            .await
    }
}
