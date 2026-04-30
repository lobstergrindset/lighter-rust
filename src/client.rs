#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::config::Config;
use crate::error::{Result, SdkError};
use crate::ffi::load_signer;
use crate::ffi::signer;
use crate::ffi::types::FfiCreateOrderTxReq;
use crate::models::transaction::{RespSendTx, RespSendTxBatch};
use crate::nonce::NonceManagerType;
use crate::nonce::api::ApiNonceManager;
use crate::nonce::manager::NonceManager;
use crate::nonce::optimistic::OptimisticNonceManager;
use crate::rest::client::LighterRestClient;
use crate::types::transact_opts::L2TxAttributes;

const CODE_OK: i64 = 200;
const CODE_INVALID_NONCE: i64 = 21104;

pub struct SignerClient {
    pub config: Config,
    pub account_index: i64,
    pub rest: LighterRestClient,
    nonce_manager: Arc<Mutex<Box<dyn NonceManager>>>,
    api_key_dict: HashMap<u8, String>,
}

impl SignerClient {
    pub async fn new(
        config: Config,
        account_index: i64,
        api_private_keys: HashMap<u8, String>,
        nonce_type: NonceManagerType,
    ) -> Result<Self> {
        if api_private_keys.is_empty() {
            return Err(SdkError::NoApiKeys);
        }

        // Trim 0x prefix from keys
        let api_key_dict: HashMap<u8, String> = api_private_keys
            .into_iter()
            .map(|(idx, key)| {
                let trimmed = key.strip_prefix("0x").unwrap_or(&key).to_string();
                (idx, trimmed)
            })
            .collect();

        // Load signer library
        info!(
            account_index,
            api_key_count = api_key_dict.len(),
            api_base_url = config.api_base_url(),
            "signer client loading signer library"
        );
        load_signer(config.signer_lib_path.as_deref())?;

        let rest = LighterRestClient::new(&config)?;

        // Create client for each api key
        for (&api_key_index, private_key) in &api_key_dict {
            info!(
                account_index,
                api_key_index,
                private_key_len = private_key.len(),
                "signer client creating key client"
            );
            signer::create_client(
                &config.api_base_url(),
                private_key,
                config.chain_id as i32,
                api_key_index as i32,
                account_index,
            )?;
        }

        // Create nonce manager
        let api_keys: Vec<u8> = api_key_dict.keys().copied().collect();
        let nonce_rest = LighterRestClient::new(&config)?;
        let nonce_manager_label = match nonce_type {
            NonceManagerType::Optimistic => "optimistic",
            NonceManagerType::Api => "api",
        };
        info!(
            account_index,
            api_keys = ?api_keys,
            nonce_manager_type = nonce_manager_label,
            "signer client initializing nonce manager"
        );

        let nonce_manager: Box<dyn NonceManager> = match nonce_type {
            NonceManagerType::Optimistic => {
                Box::new(OptimisticNonceManager::new(account_index, api_keys, nonce_rest).await?)
            }
            NonceManagerType::Api => {
                Box::new(ApiNonceManager::new(account_index, api_keys, nonce_rest))
            }
        };
        info!(
            account_index,
            nonce_manager_type = nonce_manager_label,
            "signer client initialized nonce manager"
        );

        Ok(Self {
            config,
            account_index,
            rest,
            nonce_manager: Arc::new(Mutex::new(nonce_manager)),
            api_key_dict,
        })
    }

    pub fn check_client(&self) -> Result<()> {
        for &api_key in self.api_key_dict.keys() {
            signer::check_client(api_key as i32, self.account_index)?;
        }
        Ok(())
    }

    async fn get_api_key_and_nonce(
        &self,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(u8, i64)> {
        if let (Some(key), Some(n)) = (api_key_index, nonce) {
            return Ok((key, n));
        }
        let nm = self.nonce_manager.lock().await;
        nm.next_nonce().await
    }

    async fn handle_tx_result(
        &self,
        result: &std::result::Result<RespSendTx, SdkError>,
        api_key_index: u8,
    ) {
        match result {
            Ok(resp) if resp.code == CODE_INVALID_NONCE => {
                let nm = self.nonce_manager.lock().await;
                let _ = nm.hard_refresh_nonce(api_key_index).await;
            }
            Ok(resp) if resp.code != CODE_OK => {
                let nm = self.nonce_manager.lock().await;
                nm.acknowledge_failure(api_key_index).await;
            }
            Err(e) => {
                if matches!(e, SdkError::Json(_)) {
                    // HTTP 200 but response body couldn't be parsed.
                    // The tx was processed — nonce already advanced correctly.
                    return;
                }
                let nm = self.nonce_manager.lock().await;
                let err_str = e.to_string();
                if err_str.contains("invalid nonce") {
                    let _ = nm.hard_refresh_nonce(api_key_index).await;
                } else {
                    nm.acknowledge_failure(api_key_index).await;
                }
            }
            _ => {}
        }
    }

    pub async fn create_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        order_type: i32,
        time_in_force: i32,
        reduce_only: bool,
        trigger_price: i32,
        order_expiry: i64,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.create_order_with_attributes(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type,
            time_in_force,
            reduce_only,
            trigger_price,
            order_expiry,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn create_order_with_attributes(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        order_type: i32,
        time_in_force: i32,
        reduce_only: bool,
        trigger_price: i32,
        order_expiry: i64,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let manages_nonce = api_key_index.is_none() || nonce.is_none();
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = match signer::sign_create_order_with_attributes(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask as i32,
            order_type,
            time_in_force,
            reduce_only as i32,
            trigger_price,
            order_expiry,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        ) {
            Ok(signed) => signed,
            Err(err) => {
                if manages_nonce {
                    let nm = self.nonce_manager.lock().await;
                    nm.acknowledge_failure(key).await;
                }
                return Err(err);
            }
        };
        info!(
            account_index = self.account_index,
            api_key_index = key,
            nonce = n,
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type,
            time_in_force,
            reduce_only,
            trigger_price,
            order_expiry,
            tx_type = signed.tx_type,
            tx_info_len = signed.tx_info.len(),
            "signed lighter create order"
        );
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        if let Err(error) = &result {
            warn!(
                account_index = self.account_index,
                api_key_index = key,
                nonce = n,
                market_index,
                client_order_index,
                base_amount,
                price,
                is_ask,
                order_type,
                time_in_force,
                reduce_only,
                trigger_price,
                order_expiry,
                tx_type = signed.tx_type,
                tx_info_len = signed.tx_info.len(),
                %error,
                "lighter create order send failed"
            );
        }
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn create_grouped_orders(
        &self,
        grouping_type: u8,
        orders: &[FfiCreateOrderTxReq],
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.create_grouped_orders_with_attributes(
            grouping_type,
            orders,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn create_grouped_orders_with_attributes(
        &self,
        grouping_type: u8,
        orders: &[FfiCreateOrderTxReq],
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_create_grouped_orders_with_attributes(
            grouping_type,
            orders,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn cancel_order(
        &self,
        market_index: i32,
        order_index: i64,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.cancel_order_with_attributes(
            market_index,
            order_index,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn cancel_order_with_attributes(
        &self,
        market_index: i32,
        order_index: i64,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_cancel_order_with_attributes(
            market_index,
            order_index,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn modify_order(
        &self,
        market_index: i32,
        order_index: i64,
        base_amount: i64,
        price: i64,
        trigger_price: i64,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.modify_order_with_attributes(
            market_index,
            order_index,
            base_amount,
            price,
            trigger_price,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn modify_order_with_attributes(
        &self,
        market_index: i32,
        order_index: i64,
        base_amount: i64,
        price: i64,
        trigger_price: i64,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_modify_order_with_attributes(
            market_index,
            order_index,
            base_amount,
            price,
            trigger_price,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn cancel_all_orders(
        &self,
        time_in_force: i32,
        timestamp_ms: i64,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.cancel_all_orders_with_attributes(
            time_in_force,
            timestamp_ms,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn cancel_all_orders_with_attributes(
        &self,
        time_in_force: i32,
        timestamp_ms: i64,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_cancel_all_orders_with_attributes(
            time_in_force,
            timestamp_ms,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn withdraw(
        &self,
        asset_index: i32,
        route_type: i32,
        amount: u64,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.withdraw_with_attributes(
            asset_index,
            route_type,
            amount,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn withdraw_with_attributes(
        &self,
        asset_index: i32,
        route_type: i32,
        amount: u64,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_withdraw_with_attributes(
            asset_index,
            route_type,
            amount,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn transfer(
        &self,
        to_account_index: i64,
        asset_index: i16,
        from_route_type: u8,
        to_route_type: u8,
        amount: i64,
        usdc_fee: i64,
        memo: &str,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.transfer_with_attributes(
            to_account_index,
            asset_index,
            from_route_type,
            to_route_type,
            amount,
            usdc_fee,
            memo,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn transfer_with_attributes(
        &self,
        to_account_index: i64,
        asset_index: i16,
        from_route_type: u8,
        to_route_type: u8,
        amount: i64,
        usdc_fee: i64,
        memo: &str,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_transfer_with_attributes(
            to_account_index,
            asset_index,
            from_route_type,
            to_route_type,
            amount,
            usdc_fee,
            memo,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn change_pub_key(
        &self,
        new_pub_key: &str,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.change_pub_key_with_attributes(
            new_pub_key,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn change_pub_key_with_attributes(
        &self,
        new_pub_key: &str,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_change_pub_key_with_attributes(
            new_pub_key,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn create_sub_account(
        &self,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.create_sub_account_with_attributes(L2TxAttributes::default(), api_key_index, nonce)
            .await
    }

    pub async fn create_sub_account_with_attributes(
        &self,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_create_sub_account_with_attributes(
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn create_public_pool(
        &self,
        operator_fee: i64,
        initial_total_shares: i32,
        min_operator_share_rate: i64,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.create_public_pool_with_attributes(
            operator_fee,
            initial_total_shares,
            min_operator_share_rate,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn create_public_pool_with_attributes(
        &self,
        operator_fee: i64,
        initial_total_shares: i32,
        min_operator_share_rate: i64,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_create_public_pool_with_attributes(
            operator_fee,
            initial_total_shares,
            min_operator_share_rate,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn update_public_pool(
        &self,
        public_pool_index: i64,
        status: i32,
        operator_fee: i64,
        min_operator_share_rate: i32,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.update_public_pool_with_attributes(
            public_pool_index,
            status,
            operator_fee,
            min_operator_share_rate,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn update_public_pool_with_attributes(
        &self,
        public_pool_index: i64,
        status: i32,
        operator_fee: i64,
        min_operator_share_rate: i32,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_update_public_pool_with_attributes(
            public_pool_index,
            status,
            operator_fee,
            min_operator_share_rate,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn mint_shares(
        &self,
        public_pool_index: i64,
        share_amount: i64,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.mint_shares_with_attributes(
            public_pool_index,
            share_amount,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn mint_shares_with_attributes(
        &self,
        public_pool_index: i64,
        share_amount: i64,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_mint_shares_with_attributes(
            public_pool_index,
            share_amount,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn burn_shares(
        &self,
        public_pool_index: i64,
        share_amount: i64,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.burn_shares_with_attributes(
            public_pool_index,
            share_amount,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn burn_shares_with_attributes(
        &self,
        public_pool_index: i64,
        share_amount: i64,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_burn_shares_with_attributes(
            public_pool_index,
            share_amount,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn update_leverage(
        &self,
        market_index: i32,
        initial_margin_fraction: i32,
        margin_mode: i32,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.update_leverage_with_attributes(
            market_index,
            initial_margin_fraction,
            margin_mode,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn update_leverage_with_attributes(
        &self,
        market_index: i32,
        initial_margin_fraction: i32,
        margin_mode: i32,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_update_leverage_with_attributes(
            market_index,
            initial_margin_fraction,
            margin_mode,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub async fn update_margin(
        &self,
        market_index: i32,
        usdc_amount: i64,
        direction: i32,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        self.update_margin_with_attributes(
            market_index,
            usdc_amount,
            direction,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
        .await
    }

    pub async fn update_margin_with_attributes(
        &self,
        market_index: i32,
        usdc_amount: i64,
        direction: i32,
        tx_attributes: L2TxAttributes,
        api_key_index: Option<u8>,
        nonce: Option<i64>,
    ) -> Result<(signer::SignedTx, RespSendTx)> {
        let (key, n) = self.get_api_key_and_nonce(api_key_index, nonce).await?;
        let signed = signer::sign_update_margin_with_attributes(
            market_index,
            usdc_amount,
            direction,
            n,
            key as i32,
            self.account_index,
            &tx_attributes,
        )?;
        let result = self.rest.send_tx(signed.tx_type, &signed.tx_info).await;
        self.handle_tx_result(&result, key).await;
        Ok((signed, result?))
    }

    pub fn create_auth_token(&self, deadline: i64, api_key_index: Option<u8>) -> Result<String> {
        let key = api_key_index.unwrap_or_else(|| *self.api_key_dict.keys().next().unwrap());
        signer::create_auth_token(deadline, key as i32, self.account_index)
    }

    pub async fn send_tx(&self, tx_type: u8, tx_info: &str) -> Result<RespSendTx> {
        self.rest.send_tx(tx_type, tx_info).await
    }

    pub async fn send_tx_batch(&self, tx_types: &str, tx_infos: &str) -> Result<RespSendTxBatch> {
        self.rest.send_tx_batch(tx_types, tx_infos).await
    }

    /// Reserve `count` sequential nonces on the same API key for batch operations.
    /// Returns `(api_key_index, nonces)`.
    pub async fn reserve_nonces(&self, count: usize) -> Result<(u8, Vec<i64>)> {
        let nm = self.nonce_manager.lock().await;
        let (key, first) = nm.next_nonce().await?;
        let mut nonces = vec![first];
        for _ in 1..count {
            let (_, n) = nm.next_nonce_for_key(key).await?;
            nonces.push(n);
        }
        Ok((key, nonces))
    }

    /// Sign pre-built transactions and send them as a single batch HTTP call.
    pub async fn sign_and_send_batch(
        &self,
        signed_txs: &[signer::SignedTx],
    ) -> Result<RespSendTxBatch> {
        let tx_types: Vec<u8> = signed_txs.iter().map(|t| t.tx_type).collect();
        let tx_infos: Vec<&str> = signed_txs.iter().map(|t| t.tx_info.as_str()).collect();
        let types_json = serde_json::to_string(&tx_types)?;
        let infos_json = serde_json::to_string(&tx_infos)?;
        self.rest.send_tx_batch(&types_json, &infos_json).await
    }

    /// Roll back `count` nonces for the given API key after a batch failure.
    pub async fn acknowledge_batch_failure(&self, api_key_index: u8, count: usize) {
        let nm = self.nonce_manager.lock().await;
        for _ in 0..count {
            nm.acknowledge_failure(api_key_index).await;
        }
    }

    /// Hard-refresh the nonce for a given API key (e.g. after an invalid-nonce error).
    pub async fn hard_refresh_nonce(&self, api_key_index: u8) -> Result<()> {
        let nm = self.nonce_manager.lock().await;
        nm.hard_refresh_nonce(api_key_index).await
    }

    /// Sign a create-order transaction without sending it.
    /// Use with `reserve_nonces` + `send_signed_batch` for batched submission.
    pub fn sign_create_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        order_type: i32,
        time_in_force: i32,
        reduce_only: bool,
        trigger_price: i32,
        order_expiry: i64,
        api_key_index: u8,
        nonce: i64,
    ) -> Result<signer::SignedTx> {
        self.sign_create_order_with_attributes(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            order_type,
            time_in_force,
            reduce_only,
            trigger_price,
            order_expiry,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
    }

    pub fn sign_create_order_with_attributes(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        order_type: i32,
        time_in_force: i32,
        reduce_only: bool,
        trigger_price: i32,
        order_expiry: i64,
        tx_attributes: L2TxAttributes,
        api_key_index: u8,
        nonce: i64,
    ) -> Result<signer::SignedTx> {
        signer::sign_create_order_with_attributes(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask as i32,
            order_type,
            time_in_force,
            reduce_only as i32,
            trigger_price,
            order_expiry,
            nonce,
            api_key_index as i32,
            self.account_index,
            &tx_attributes,
        )
    }

    /// Sign a cancel-order transaction without sending it.
    /// Use with `reserve_nonces` + `send_signed_batch` for batched submission.
    pub fn sign_cancel_order(
        &self,
        market_index: i32,
        order_index: i64,
        api_key_index: u8,
        nonce: i64,
    ) -> Result<signer::SignedTx> {
        self.sign_cancel_order_with_attributes(
            market_index,
            order_index,
            L2TxAttributes::default(),
            api_key_index,
            nonce,
        )
    }

    pub fn sign_cancel_order_with_attributes(
        &self,
        market_index: i32,
        order_index: i64,
        tx_attributes: L2TxAttributes,
        api_key_index: u8,
        nonce: i64,
    ) -> Result<signer::SignedTx> {
        signer::sign_cancel_order_with_attributes(
            market_index,
            order_index,
            nonce,
            api_key_index as i32,
            self.account_index,
            &tx_attributes,
        )
    }

    /// Send pre-signed transactions as a single batch.
    /// Handles nonce error detection and rollback.
    pub async fn send_signed_batch(
        &self,
        signed_txs: &[signer::SignedTx],
        api_key_index: u8,
    ) -> Result<RespSendTxBatch> {
        let tx_types: Vec<u8> = signed_txs.iter().map(|t| t.tx_type).collect();
        let tx_infos: Vec<&str> = signed_txs.iter().map(|t| t.tx_info.as_str()).collect();
        let types_json = serde_json::to_string(&tx_types)?;
        let infos_json = serde_json::to_string(&tx_infos)?;
        let result = self.rest.send_tx_batch(&types_json, &infos_json).await;

        match &result {
            Ok(resp) if resp.code == CODE_INVALID_NONCE => {
                let nm = self.nonce_manager.lock().await;
                let _ = nm.hard_refresh_nonce(api_key_index).await;
            }
            Ok(resp) if resp.code != CODE_OK => {
                self.acknowledge_batch_failure(api_key_index, signed_txs.len())
                    .await;
            }
            Err(_) => {
                self.acknowledge_batch_failure(api_key_index, signed_txs.len())
                    .await;
            }
            _ => {}
        }

        result
    }
}
