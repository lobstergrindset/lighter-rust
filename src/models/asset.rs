use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub symbol: String,
    pub asset_id: i64,
    pub balance: String,
    pub locked_balance: String,
}

impl Asset {
    pub fn balance_f64(&self) -> Option<f64> {
        self.balance.parse::<f64>().ok()
    }

    pub fn locked_balance_f64(&self) -> Option<f64> {
        self.locked_balance.parse::<f64>().ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDetails {
    pub code: i64,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    #[serde(alias = "assets")]
    pub asset_details: Vec<Asset>,
}
