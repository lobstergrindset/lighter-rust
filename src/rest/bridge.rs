use crate::error::Result;
use crate::models::bridge::*;
use crate::rest::client::LighterRestClient;

impl LighterRestClient {
    pub async fn create_intent_address(
        &self,
        chain_id: u64,
        from_addr: &str,
        amount: u64,
        is_external_deposit: bool,
    ) -> Result<IntentAddress> {
        let chain_id = chain_id.to_string();
        let amount = amount.to_string();
        let is_external_deposit = is_external_deposit.to_string();
        self.post_form(
            "/api/v1/createIntentAddress",
            &[
                ("chain_id", chain_id.as_str()),
                ("from_addr", from_addr),
                ("amount", amount.as_str()),
                ("is_external_deposit", is_external_deposit.as_str()),
            ],
        )
        .await
    }

    pub async fn get_fast_withdraw_info(
        &self,
        account_index: i64,
        auth: &str,
    ) -> Result<FastWithdrawInfo> {
        let account_index = account_index.to_string();
        self.get_with_auth(
            "/api/v1/fastwithdraw/info",
            &[("account_index", account_index.as_str())],
            auth,
        )
        .await
    }

    pub async fn get_transfer_fee_info(
        &self,
        account_index: i64,
        to_account_index: i64,
        auth: &str,
    ) -> Result<TransferFeeInfo> {
        let account_index = account_index.to_string();
        let to_account_index = to_account_index.to_string();
        self.get_with_auth(
            "/api/v1/transferFeeInfo",
            &[
                ("account_index", account_index.as_str()),
                ("to_account_index", to_account_index.as_str()),
            ],
            auth,
        )
        .await
    }

    pub async fn fast_withdraw(
        &self,
        tx_info: &str,
        to_address: &str,
        auth: &str,
    ) -> Result<crate::models::transaction::RespSendTx> {
        self.post_form_with_auth(
            "/api/v1/fastwithdraw",
            &[("tx_info", tx_info), ("to_address", to_address)],
            auth,
        )
        .await
    }

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
