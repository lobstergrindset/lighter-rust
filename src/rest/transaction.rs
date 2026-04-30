use crate::error::Result;
use crate::models::account::NextNonce;
use crate::models::transaction::*;
use crate::rest::client::LighterRestClient;
use tracing::{info, warn};

impl LighterRestClient {
    pub async fn send_tx(&self, tx_type: u8, tx_info: &str) -> Result<RespSendTx> {
        info!(
            tx_type,
            tx_info_len = tx_info.len(),
            "sending lighter tx via multipart form"
        );
        let tx_type = tx_type.to_string();
        let result: Result<RespSendTx> = self
            .post_multipart(
                "/api/v1/sendTx",
                &[("tx_type", tx_type.as_str()), ("tx_info", tx_info)],
            )
            .await;
        match &result {
            Ok(response) => info!(
                code = response.code,
                message = response.message.as_deref().unwrap_or(""),
                tx_hash = response.tx_hash.as_deref().unwrap_or(""),
                predicted_execution_time_ms = response.predicted_execution_time_ms,
                volume_quota_remaining = response.volume_quota_remaining,
                "lighter sendTx response"
            ),
            Err(error) => warn!(%error, "lighter sendTx request failed"),
        }
        result
    }

    pub async fn send_tx_batch(&self, tx_types: &str, tx_infos: &str) -> Result<RespSendTxBatch> {
        info!(
            tx_types_len = tx_types.len(),
            tx_infos_len = tx_infos.len(),
            "sending lighter tx batch via multipart form"
        );
        let result: Result<RespSendTxBatch> = self
            .post_multipart(
                "/api/v1/sendTxBatch",
                &[("tx_types", tx_types), ("tx_infos", tx_infos)],
            )
            .await;
        match &result {
            Ok(response) => info!(
                code = response.code,
                message = response.message.as_deref().unwrap_or(""),
                tx_hashes = ?response.tx_hash,
                predicted_execution_time_ms = response.predicted_execution_time_ms,
                volume_quota_remaining = response.volume_quota_remaining,
                "lighter sendTxBatch response"
            ),
            Err(error) => warn!(%error, "lighter sendTxBatch request failed"),
        }
        result
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
