use crate::error::Result;
use crate::models::account::*;
use crate::rest::client::LighterRestClient;

impl LighterRestClient {
    /// Look up accounts by a given field (`"l1_address"`, `"index"`, etc.).
    pub async fn get_account(&self, by: &str, value: &str) -> Result<Accounts> {
        self.get_with_query("/api/v1/account", &[("by", by), ("value", value)])
            .await
    }

    /// Look up account by numeric account index.
    pub async fn get_account_by_index(&self, account_index: i64) -> Result<Accounts> {
        let account_index = account_index.to_string();
        self.get_account("index", account_index.as_str()).await
    }

    /// Look up accounts with an auth header (returns positions, PnL, etc.).
    pub async fn get_detailed_account(
        &self,
        by: &str,
        value: &str,
        auth: &str,
    ) -> Result<DetailedAccounts> {
        self.get_with_auth("/api/v1/account", &[("by", by), ("value", value)], auth)
            .await
    }

    /// Look up account details by numeric account index.
    pub async fn get_detailed_account_by_index(
        &self,
        account_index: i64,
        auth: &str,
    ) -> Result<DetailedAccounts> {
        let account_index = account_index.to_string();
        self.get_detailed_account("index", account_index.as_str(), auth)
            .await
    }

    pub async fn get_account_api_keys(
        &self,
        account_index: i64,
        auth: &str,
    ) -> Result<AccountApiKeys> {
        let account_index = account_index.to_string();
        self.get_with_auth(
            "/api/v1/apikeys",
            &[("account_index", account_index.as_str())],
            auth,
        )
        .await
    }

    pub async fn get_account_limits(
        &self,
        account_index: i64,
        auth: &str,
    ) -> Result<AccountLimits> {
        let account_index = account_index.to_string();
        self.get_with_auth(
            "/api/v1/accountLimits",
            &[("account_index", account_index.as_str())],
            auth,
        )
        .await
    }

    pub async fn get_account_metadata(
        &self,
        by: &str,
        value: &str,
        auth: &str,
    ) -> Result<AccountMetadatas> {
        self.get_with_auth(
            "/api/v1/accountMetadata",
            &[("by", by), ("value", value)],
            auth,
        )
        .await
    }

    pub async fn get_account_metadata_by_index(
        &self,
        account_index: i64,
        auth: &str,
    ) -> Result<AccountMetadatas> {
        let account_index = account_index.to_string();
        self.get_account_metadata("index", account_index.as_str(), auth)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_account_pnl(
        &self,
        account_index: i64,
        resolution: &str,
        start_timestamp: i64,
        end_timestamp: i64,
        count_back: i64,
        ignore_transfers: bool,
        auth: &str,
    ) -> Result<AccountPnl> {
        let account_index = account_index.to_string();
        let start_timestamp = start_timestamp.to_string();
        let end_timestamp = end_timestamp.to_string();
        let count_back = count_back.to_string();
        let mut query = vec![
            ("by", "index"),
            ("value", account_index.as_str()),
            ("resolution", resolution),
            ("start_timestamp", start_timestamp.as_str()),
            ("end_timestamp", end_timestamp.as_str()),
            ("count_back", count_back.as_str()),
        ];
        if ignore_transfers {
            query.push(("ignore_transfers", "true"));
        }
        self.get_with_auth("/api/v1/pnl", &query, auth).await
    }

    pub async fn get_sub_accounts(&self, l1_address: &str) -> Result<SubAccounts> {
        self.get_with_query("/api/v1/accountsByL1Address", &[("l1_address", l1_address)])
            .await
    }

    /// Change the account tier (e.g. "standard" -> "premium").
    pub async fn change_account_tier(
        &self,
        account_index: i64,
        new_tier: &str,
        auth: &str,
    ) -> Result<ChangeAccountTierResponse> {
        let account_index = account_index.to_string();
        self.post_form_with_auth(
            "/api/v1/changeAccountTier",
            &[
                ("account_index", account_index.as_str()),
                ("new_tier", new_tier),
            ],
            auth,
        )
        .await
    }
}
