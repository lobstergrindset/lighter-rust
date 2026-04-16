use thiserror::Error;

use crate::constants::*;

#[derive(Debug, Error)]
pub enum SdkError {
    // === Validation errors (from Go txtypes/errors.go) ===
    #[error("AssetIndex should not be less than {MIN_ASSET_INDEX}")]
    AssetIndexTooLow,
    #[error("AssetIndex should not be larger than {MAX_ASSET_INDEX}")]
    AssetIndexTooHigh,
    #[error("RouteType is invalid")]
    RouteTypeInvalid,
    #[error("AccountIndex should not be less than {MIN_ACCOUNT_INDEX}")]
    AccountIndexTooLow,
    #[error("AccountIndex should not be larger than {MAX_ACCOUNT_INDEX}")]
    AccountIndexTooHigh,
    #[error("AccountNonce should not be less than {MIN_NONCE}")]
    NonceTooLow,
    #[error("AccountNonce should be less than {MAX_NONCE_EXCLUSIVE}")]
    NonceTooHigh,
    #[error("CancelAllTimeInForce is invalid")]
    InvalidCancelAllTimeInForce,
    #[error("ReduceOnly is invalid")]
    OrderReduceOnlyInvalid,
    #[error("TriggerPrice is invalid")]
    OrderTriggerPriceInvalid,
    #[error("OrderExpiry is invalid")]
    OrderExpiryInvalid,
    #[error("ExpiredAt is invalid")]
    ExpiredAtInvalid,
    #[error("CancelAllTime should be larger than 0 and not larger than {MAX_ORDER_EXPIRY}")]
    CancelAllTimeIsNotInRange,
    #[error("CancelAllTime should be nil")]
    CancelAllTimeIsNotNil,
    #[error("PubKey is invalid")]
    PubKeyInvalid,
    #[error("ToAccountIndex should not be less than {MIN_ACCOUNT_INDEX}")]
    ToAccountIndexTooLow,
    #[error("ToAccountIndex should not be larger than {MAX_ACCOUNT_INDEX}")]
    ToAccountIndexTooHigh,
    #[error("FromAccountIndex should not be less than {MIN_ACCOUNT_INDEX}")]
    FromAccountIndexTooLow,
    #[error("FromAccountIndex should not be larger than {MAX_ACCOUNT_INDEX}")]
    FromAccountIndexTooHigh,
    #[error("ApiKeyIndex should not be less than {MIN_API_KEY_INDEX}")]
    ApiKeyIndexTooLow,
    #[error("ApiKeyIndex should not be larger than {MAX_API_KEY_INDEX}")]
    ApiKeyIndexTooHigh,
    #[error("PublicPoolIndex should not be less than {MIN_ACCOUNT_INDEX}")]
    PublicPoolIndexTooLow,
    #[error("PublicPoolIndex should not be larger than {MAX_ACCOUNT_INDEX}")]
    PublicPoolIndexTooHigh,
    #[error("PoolOperatorFee should be larger than 0 and not larger than {FEE_TICK}")]
    InvalidPoolOperatorFee,
    #[error("PoolStatus should be either 0 or 1")]
    InvalidPoolStatus,
    #[error("PoolInitialTotalShares should be larger than {MIN_INITIAL_TOTAL_SHARES}")]
    PoolInitialTotalSharesTooLow,
    #[error("PoolInitialTotalShares should not be larger than {MAX_INITIAL_TOTAL_SHARES}")]
    PoolInitialTotalSharesTooHigh,
    #[error("PoolMinOperatorShareRate should be larger than 0")]
    PoolMinOperatorShareRateTooLow,
    #[error("PoolMinOperatorShareRate should not be larger than {SHARE_TICK}")]
    PoolMinOperatorShareRateTooHigh,
    #[error("PoolMintShareAmount should be larger than {MIN_POOL_SHARES_TO_MINT_OR_BURN}")]
    PoolMintShareAmountTooLow,
    #[error("PoolMintShareAmount should not be larger than {MAX_POOL_SHARES_TO_MINT_OR_BURN}")]
    PoolMintShareAmountTooHigh,
    #[error("PoolBurnShareAmount should be larger than {MIN_POOL_SHARES_TO_MINT_OR_BURN}")]
    PoolBurnShareAmountTooLow,
    #[error("PoolBurnShareAmount should not be larger than {MAX_POOL_SHARES_TO_MINT_OR_BURN}")]
    PoolBurnShareAmountTooHigh,
    #[error("WithdrawalAmount should be larger than {MIN_WITHDRAWAL_AMOUNT}")]
    WithdrawalAmountTooLow,
    #[error("WithdrawalAmount should not be larger than {MAX_WITHDRAWAL_AMOUNT}")]
    WithdrawalAmountTooHigh,
    #[error("TransferAmount should be larger than {MIN_TRANSFER_AMOUNT}")]
    TransferAmountTooLow,
    #[error("TransferAmount should not be larger than {MAX_TRANSFER_AMOUNT}")]
    TransferAmountTooHigh,
    #[error("TransferFee should not be negative")]
    TransferFeeNegative,
    #[error("TransferFee should not be larger than {MAX_TRANSFER_AMOUNT}")]
    TransferFeeTooHigh,
    #[error("MarketIndex should not be less than {MIN_MARKET_INDEX}")]
    MarketIndexTooLow,
    #[error("MarketIndex should not be larger than {MAX_SPOT_MARKET_INDEX}")]
    MarketIndexTooHigh,
    #[error("MarketIndex should match the market index of the order")]
    MarketIndexMismatch,
    #[error("MarketIndex is not valid")]
    InvalidMarketIndex,
    #[error("InitialMarginFraction should not be less than 0")]
    InitialMarginFractionTooLow,
    #[error("InitialMarginFraction should not be larger than {MARGIN_FRACTION_TICK}")]
    InitialMarginFractionTooHigh,
    #[error("ClientOrderIndex should not be less than {MIN_CLIENT_ORDER_INDEX}")]
    ClientOrderIndexTooLow,
    #[error("ClientOrderIndex should not be larger than {MAX_CLIENT_ORDER_INDEX}")]
    ClientOrderIndexTooHigh,
    #[error("ClientOrderIndex should be nil")]
    ClientOrderIndexNotNil,
    #[error("OrderIndex should not be less than {MIN_ORDER_INDEX}")]
    OrderIndexTooLow,
    #[error("OrderIndex should not be larger than {MAX_ORDER_INDEX}")]
    OrderIndexTooHigh,
    #[error("BaseAmount should not be less than {MIN_ORDER_BASE_AMOUNT}")]
    BaseAmountTooLow,
    #[error("BaseAmount should not be larger than {MAX_ORDER_BASE_AMOUNT}")]
    BaseAmountTooHigh,
    #[error("BaseAmounts should be equal")]
    BaseAmountsNotEqual,
    #[error("BaseAmount should be nil")]
    BaseAmountNotNil,
    #[error("OrderPrice should not be less than {MIN_ORDER_PRICE}")]
    PriceTooLow,
    #[error("OrderPrice should not be larger than {MAX_ORDER_PRICE}")]
    PriceTooHigh,
    #[error("IsAsk should be 0 or 1")]
    IsAskInvalid,
    #[error("OrderType is not valid")]
    OrderTypeInvalid,
    #[error("OrderTimeInForce is not valid")]
    OrderTimeInForceInvalid,
    #[error("GroupingType is not valid")]
    GroupingTypeInvalid,
    #[error("OrderGroupSize is not valid")]
    OrderGroupSizeInvalid,
    #[error("TxSignature is invalid")]
    InvalidSignature,
    #[error("MarginMode is not valid")]
    InvalidMarginMode,
    #[error("CancelMode is not valid")]
    CancelModeInvalid,
    #[error("Margin movement direction is not valid")]
    InvalidUpdateMarginDirection,
    #[error("Nonce skip attribute is invalid")]
    NonceSkipAttributeInvalid,
    #[error("IntegratorAccountIndex is in invalid range")]
    IntegratorAccountIndexInvalidRange,
    #[error("Integrator fees are in invalid range")]
    IntegratorFeeInvalidRange,
    #[error(
        "IntegratorAccountIndex should be non-zero when integrator taker fee or maker fee is non-zero"
    )]
    IntegratorAccountIndexRequiredForNonZeroFees,

    // === FFI errors ===
    #[error("FFI error: {0}")]
    Ffi(String),
    #[error("Signer library not loaded")]
    SignerNotLoaded,
    #[error(
        "Failed to load signer library. Configure `Config::with_signer_lib_path`, set `LIGHTER_SIGNER_LIB_PATH`, or place the signer next to your executable/current directory. Last error: {0}"
    )]
    SignerLoadFailed(String),

    // === HTTP/REST errors ===
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error (code {code}): {message}")]
    Api { code: i64, message: String },
    #[error("Rate limited (HTTP {code}): {message}")]
    RateLimited { code: i64, message: String },

    // === WebSocket errors ===
    #[error("WebSocket error: {0}")]
    WebSocket(Box<tokio_tungstenite::tungstenite::Error>),
    #[error("WebSocket not connected")]
    WebSocketNotConnected,

    // === Serialization errors ===
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    // === Other ===
    #[error("No API keys provided")]
    NoApiKeys,
    #[error("Ambiguous API key")]
    AmbiguousApiKey,
    #[error("{0}")]
    Other(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for SdkError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(Box::new(err))
    }
}

pub type Result<T> = std::result::Result<T, SdkError>;

impl SdkError {
    pub fn is_rate_limited(&self) -> bool {
        match self {
            Self::RateLimited { .. } => true,
            Self::Api { code, .. } => matches!(*code, 405 | 429),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SdkError;

    #[test]
    fn classifies_new_rate_limited_variant() {
        let err = SdkError::RateLimited {
            code: 405,
            message: "cooldown".to_string(),
        };

        assert!(err.is_rate_limited());
    }

    #[test]
    fn classifies_legacy_api_status_codes_for_rate_limits() {
        let too_many = SdkError::Api {
            code: 429,
            message: "too many requests".to_string(),
        };
        let method_not_allowed = SdkError::Api {
            code: 405,
            message: "method not allowed".to_string(),
        };
        let other = SdkError::Api {
            code: 500,
            message: "server error".to_string(),
        };

        assert!(too_many.is_rate_limited());
        assert!(method_not_allowed.is_rate_limited());
        assert!(!other.is_rate_limited());
    }
}
