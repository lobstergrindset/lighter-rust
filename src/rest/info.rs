use crate::error::Result;
use crate::models::asset::AssetDetails;
use crate::models::info::*;
use crate::rest::client::LighterRestClient;

impl LighterRestClient {
    pub async fn get_exchange_stats(&self) -> Result<ExchangeStats> {
        self.get("/api/v1/exchangeStats").await
    }

    pub async fn get_asset_details(&self) -> Result<AssetDetails> {
        self.get("/api/v1/assetDetails").await
    }

    pub async fn get_zk_lighter_info(&self) -> Result<ValidatorInfo> {
        self.get("/info").await
    }
}
