use crate::error::Result;
use crate::models::account::NextNonce;
use crate::models::transaction::*;
use crate::rest::client::LighterRestClient;

impl LighterRestClient {
    pub async fn send_tx(&self, tx_type: u8, tx_info: &str) -> Result<RespSendTx> {
        self.post_form(
            "/api/v1/sendTx",
            &[("tx_type", &tx_type.to_string()), ("tx_info", tx_info)],
        )
        .await
    }

    pub async fn send_tx_batch(&self, tx_types: &str, tx_infos: &str) -> Result<RespSendTxBatch> {
        self.post_form(
            "/api/v1/sendTxBatch",
            &[("tx_types", tx_types), ("tx_infos", tx_infos)],
        )
        .await
    }

    pub async fn get_next_nonce(&self, account_index: i64, api_key_index: u8) -> Result<NextNonce> {
        let account_index = account_index.to_string();
        let api_key_index = api_key_index.to_string();
        self.get_with_query(
            "/api/v1/nextNonce",
            &[
                ("account_index", account_index.as_str()),
                ("api_key_index", api_key_index.as_str()),
            ],
        )
        .await
    }

    pub async fn get_enriched_tx(&self, tx_hash: &str) -> Result<EnrichedTx> {
        self.get_with_query("/api/v1/tx", &[("tx_hash", tx_hash)])
            .await
    }
}
