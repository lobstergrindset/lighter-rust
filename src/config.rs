use crate::constants::{MAINNET_CHAIN_ID, TESTNET_CHAIN_ID};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub chain_id: u32,
    pub use_ssl: bool,
    pub pool_size: usize,
    pub signer_lib_path: Option<String>,
    pub proxy: Option<String>,
}

impl Config {
    pub fn new(host: impl Into<String>) -> Self {
        let host = host.into();
        let chain_id = if host.contains("mainnet") {
            MAINNET_CHAIN_ID
        } else {
            TESTNET_CHAIN_ID
        };
        Self {
            host,
            chain_id,
            use_ssl: true,
            pool_size: 10,
            signer_lib_path: None,
            proxy: None,
        }
    }

    pub fn with_chain_id(mut self, chain_id: u32) -> Self {
        self.chain_id = chain_id;
        self
    }

    pub fn with_ssl(mut self, use_ssl: bool) -> Self {
        self.use_ssl = use_ssl;
        self
    }

    pub fn with_pool_size(mut self, pool_size: usize) -> Self {
        self.pool_size = pool_size;
        self
    }

    /// Accepts either a directory containing the signer shared library or a
    /// full path to the library file itself.
    pub fn with_signer_lib_path(mut self, path: impl Into<String>) -> Self {
        self.signer_lib_path = Some(path.into());
        self
    }

    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    pub fn api_base_url(&self) -> String {
        let scheme = if self.use_ssl { "https" } else { "http" };
        format!("{scheme}://{}", self.host)
    }

    pub fn ws_base_url(&self) -> String {
        let scheme = if self.use_ssl { "wss" } else { "ws" };
        format!("{scheme}://{}/stream", self.host)
    }
}
