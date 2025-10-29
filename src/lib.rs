pub mod abi;
pub mod events;
pub mod global;
pub mod liquidity;
pub mod price;
pub mod tool;
pub mod types;
use crate::abi::{IPool, IPoolManager};
use crate::analyze::FarmAnalyzer;
use crate::events::{FarmEventListener, UniswapEventListener};
use crate::farm::FarmService;
use crate::router::Router;
use crate::tool::address::str_to_address;
use crate::tool::cal_price_from_sqrt_price_x96;
use crate::types::{EvmType, PairCreatedEvent, SwapEvent, TickInfo, V4PoolInfo, V4PositionInfo};
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Bytes;
use ethers::types::{Address, H256, U256};
use std::sync::Arc;
pub mod analyze;
pub mod factory;
pub mod farm;
pub mod oracle;
pub mod risk;
pub mod router;
use crate::types::EvmError;

/// EVM Client for interacting with various EVM chains
#[derive(Clone)]
pub struct EvmClient {
    pub provider: Arc<Provider<Http>>,
    pub chain: EvmType,
    pub wallet: Option<LocalWallet>,
}

impl EvmClient {
    /// Create a new EVM client without wallet
    pub async fn new(chain: EvmType) -> Result<Self, EvmError> {
        let rpc_url = match chain {
            EvmType::Ethereum => global::rpc::ETHEREUM_RPC,
            EvmType::Arb => global::rpc::ARB_RPC,
            EvmType::Bsc => global::rpc::BSC_RPC,
            EvmType::Base => global::rpc::BASE_RPC,
            EvmType::HyperEVM => global::rpc::HYPEREVM_RPC,
            EvmType::Plasma => global::rpc::PLASMA_RPC,
        };
        if rpc_url.is_empty() {
            return Err(EvmError::ConfigError("RPC URL not configured".to_string()));
        }
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| EvmError::ConnectionError(format!("Failed to connect to RPC: {}", e)))?;
        Ok(Self {
            provider: Arc::new(provider),
            chain,
            wallet: None,
        })
    }

    /// Create a new EVM client with wallet
    pub async fn with_wallet(chain: EvmType, private_key: &str) -> Result<Self, EvmError> {
        let rpc_url = match chain {
            EvmType::Ethereum => global::rpc::ETHEREUM_RPC,
            EvmType::Arb => global::rpc::ARB_RPC,
            EvmType::Bsc => global::rpc::BSC_RPC,
            EvmType::Base => global::rpc::BASE_RPC,
            EvmType::HyperEVM => global::rpc::HYPEREVM_RPC,
            EvmType::Plasma => global::rpc::PLASMA_RPC,
        };
        if rpc_url.is_empty() {
            return Err(EvmError::ConfigError("RPC URL not configured".to_string()));
        }
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| EvmError::ConnectionError(format!("Failed to connect to RPC: {}", e)))?;
        let wallet: LocalWallet = private_key
            .parse()
            .map_err(|e| EvmError::WalletError(format!("Failed to parse private key: {}", e)))?;
        Ok(Self {
            provider: Arc::new(provider),
            chain,
            wallet: Some(wallet),
        })
    }

    /// Get logs by filter
    pub async fn get_logs(
        &self,
        filter: ethers::types::Filter,
    ) -> Result<Vec<ethers::types::Log>, EvmError> {
        self.provider
            .get_logs(&filter)
            .await
            .map_err(|e| EvmError::RpcError(format!("Failed to get logs: {}", e)))
    }

    /// Get block number
    pub async fn get_block_number(&self) -> Result<u64, EvmError> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| EvmError::RpcError(format!("Failed to get block number: {}", e)))
            .map(|num| num.as_u64())
    }
}

/// Service for interacting with Uniswap V2 and V3 protocols
pub struct UniswapService {
    client: Arc<EvmClient>,
    router: Router,
}

impl UniswapService {
    /// Create a new UniswapService instance
    pub fn new(client: Arc<EvmClient>) -> Self {
        Self {
            client: client.clone(),
            router: Router::new(client),
        }
    }

    /// Get farm service instance
    pub fn farm_service(&self) -> FarmService {
        FarmService::new(self.client.clone())
    }

    /// Get farm analyzer instance
    pub fn farm_analyzer(&self) -> FarmAnalyzer {
        FarmAnalyzer::new(self.client.clone())
    }

    /// Get farm event listener instance
    pub fn farm_event_listener(&self) -> FarmEventListener {
        FarmEventListener::new(self.client.clone())
    }

    /// Get the output amounts for a given input amount along a specified path
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let amounts = uniswap_service.get_amounts_out(
    ///     router_address,
    ///     U256::from(1000000),
    ///     vec![token_a, token_b]
    /// ).await?;
    /// ```
    pub async fn get_amounts_out(
        &self,
        router_address: Address,
        amount_in: U256,
        path: Vec<Address>,
    ) -> Result<Vec<U256>, EvmError> {
        let router = self.router.v2_router(router_address);
        router
            .get_amounts_out(amount_in, path)
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get amounts out: {}", e)))
    }

    /// Get the input amounts for a given output amount along a specified path
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let amounts = uniswap_service.get_amounts_in(
    ///     router_address,
    ///     U256::from(1000000),
    ///     vec![token_a, token_b]
    /// ).await?;
    /// ```
    pub async fn get_amounts_in(
        &self,
        router_address: Address,
        amount_out: U256,
        path: Vec<Address>,
    ) -> Result<Vec<U256>, EvmError> {
        let router = self.router.v2_router(router_address);
        router
            .get_amounts_in(amount_out, path)
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get amounts in: {}", e)))
    }

    /// Swap exact tokens for tokens
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let tx_hash = uniswap_service.swap_exact_tokens_for_tokens(
    ///     router_address,
    ///     U256::from(1000000),
    ///     U256::from(900000),
    ///     vec![token_a, token_b],
    ///     1698765432
    /// ).await?;
    /// ```
    pub async fn swap_exact_tokens_for_tokens(
        &self,
        router_address: Address,
        amount_in: U256,
        amount_out_min: U256,
        path: Vec<Address>,
        deadline: u64,
    ) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let wallet_address = self.client.wallet.as_ref().unwrap().address();
        let router = self.router.v2_router(router_address);
        let tx = router.swap_exact_tokens_for_tokens(
            amount_in,
            amount_out_min,
            path,
            wallet_address,
            deadline.into(),
        );
        let pending_tx = tx
            .send()
            .await
            .map_err(|e| EvmError::TransactionError(format!("Failed to swap tokens: {}", e)))?;
        Ok(pending_tx.tx_hash())
    }

    /// Swap exact ETH for tokens
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let tx_hash = uniswap_service.swap_exact_eth_for_tokens(
    ///     router_address,
    ///     U256::from(900000),
    ///     vec![weth_address, token_b],
    ///     U256::from(1000000000000000000u64),
    ///     1698765432
    /// ).await?;
    /// ```
    pub async fn swap_exact_eth_for_tokens(
        &self,
        router_address: Address,
        amount_out_min: U256,
        path: Vec<Address>,
        value: U256,
        deadline: u64,
    ) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let wallet_address = self.client.wallet.as_ref().unwrap().address();
        let router = self.router.v2_router(router_address);
        let tx = router
            .swap_exact_eth_for_tokens(amount_out_min, path, wallet_address, deadline.into())
            .value(value);
        let pending_tx = tx.send().await.map_err(|e| {
            EvmError::TransactionError(format!("Failed to swap ETH for tokens: {}", e))
        })?;
        Ok(pending_tx.tx_hash())
    }

    /// Swap exact tokens for ETH
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let tx_hash = uniswap_service.swap_exact_tokens_for_eth(
    ///     router_address,
    ///     U256::from(1000000),
    ///     U256::from(900000000000000000u64),
    ///     vec![token_a, weth_address],
    ///     1698765432
    /// ).await?;
    /// ```
    pub async fn swap_exact_tokens_for_eth(
        &self,
        router_address: Address,
        amount_in: U256,
        amount_out_min: U256,
        path: Vec<Address>,
        deadline: u64,
    ) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let wallet_address = self.client.wallet.as_ref().unwrap().address();
        let router = self.router.v2_router(router_address);
        let tx = router.swap_exact_tokens_for_eth(
            amount_in,
            amount_out_min,
            path,
            wallet_address,
            deadline.into(),
        );
        let pending_tx = tx.send().await.map_err(|e| {
            EvmError::TransactionError(format!("Failed to swap tokens for ETH: {}", e))
        })?;
        Ok(pending_tx.tx_hash())
    }

    /// Add liquidity to a token pair
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let tx_hash = uniswap_service.add_liquidity(
    ///     router_address,
    ///     token_a,
    ///     token_b,
    ///     U256::from(1000000),
    ///     U256::from(2000000),
    ///     U256::from(900000),
    ///     U256::from(1800000),
    ///     1698765432
    /// ).await?;
    /// ```
    pub async fn add_liquidity(
        &self,
        router_address: Address,
        token_a: Address,
        token_b: Address,
        amount_a_desired: U256,
        amount_b_desired: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        deadline: u64,
    ) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let wallet_address = self.client.wallet.as_ref().unwrap().address();
        let router = self.router.v2_router(router_address);
        let tx = router.add_liquidity(
            token_a,
            token_b,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
            wallet_address,
            deadline.into(),
        );
        let pending_tx = tx
            .send()
            .await
            .map_err(|e| EvmError::TransactionError(format!("Failed to add liquidity: {}", e)))?;
        Ok(pending_tx.tx_hash())
    }

    /// Add liquidity with ETH
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let tx_hash = uniswap_service.add_liquidity_eth(
    ///     router_address,
    ///     token_a,
    ///     U256::from(1000000),
    ///     U256::from(900000),
    ///     U256::from(1000000000000000000u64),
    ///     U256::from(1000000000000000000u64),
    ///     1698765432
    /// ).await?;
    /// ```
    pub async fn add_liquidity_eth(
        &self,
        router_address: Address,
        token: Address,
        amount_token_desired: U256,
        amount_token_min: U256,
        amount_eth_min: U256,
        value: U256,
        deadline: u64,
    ) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let wallet_address = self.client.wallet.as_ref().unwrap().address();
        let router = self.router.v2_router(router_address);
        let tx = router
            .add_liquidity_eth(
                token,
                amount_token_desired,
                amount_token_min,
                amount_eth_min,
                wallet_address,
                deadline.into(),
            )
            .value(value);
        let pending_tx = tx.send().await.map_err(|e| {
            EvmError::TransactionError(format!("Failed to add liquidity with ETH: {}", e))
        })?;
        Ok(pending_tx.tx_hash())
    }

    /// Execute exact input single swap on Uniswap V3
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let tx_hash = uniswap_service.exact_input_single(
    ///     router_address,
    ///     token_a,
    ///     token_b,
    ///     3000,
    ///     U256::from(1000000),
    ///     U256::from(900000),
    ///     U256::from(0),
    ///     1698765432
    /// ).await?;
    /// ```
    pub async fn exact_input_single(
        &self,
        router_address: Address,
        token_in: Address,
        token_out: Address,
        fee: u32,
        amount_in: U256,
        amount_out_min: U256,
        sqrt_price_limit_x96: U256,
        deadline: u64,
    ) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let wallet_address = self.client.wallet.as_ref().unwrap().address();
        let router = self.router.v3_router(router_address);
        let tx = router.exact_input_single(
            token_in,
            token_out,
            fee,
            wallet_address,
            deadline.into(),
            amount_in,
            amount_out_min,
            sqrt_price_limit_x96.into(),
        );
        let pending_tx = tx.send().await.map_err(|e| {
            EvmError::TransactionError(format!("Failed to execute exact input single: {}", e))
        })?;
        Ok(pending_tx.tx_hash())
    }

    /// Execute exact input swap with path on Uniswap V3
    pub async fn exact_input(
        &self,
        router_address: Address,
        path: Bytes,
        amount_in: U256,
        amount_out_min: U256,
        deadline: u64,
    ) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let wallet_address = self.client.wallet.as_ref().unwrap().address();
        let router = self.router.v3_router(router_address);
        let tx = router.exact_input(
            path,
            wallet_address,
            deadline.into(),
            amount_in,
            amount_out_min,
        );
        let pending_tx = tx.send().await.map_err(|e| {
            EvmError::TransactionError(format!("Failed to execute exact input: {}", e))
        })?;
        Ok(pending_tx.tx_hash())
    }

    /// Execute exact output single swap on Uniswap V3
    pub async fn exact_output_single(
        &self,
        router_address: Address,
        token_in: Address,
        token_out: Address,
        fee: u32,
        amount_out: U256,
        amount_in_max: U256,
        sqrt_price_limit_x96: U256,
        deadline: u64,
    ) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let wallet_address = self.client.wallet.as_ref().unwrap().address();
        let router = self.router.v3_router(router_address);
        let tx = router.exact_output_single(
            token_in,
            token_out,
            fee,
            wallet_address,
            deadline.into(),
            amount_out,
            amount_in_max,
            sqrt_price_limit_x96.into(),
        );
        let pending_tx = tx.send().await.map_err(|e| {
            EvmError::TransactionError(format!("Failed to execute exact output single: {}", e))
        })?;
        Ok(pending_tx.tx_hash())
    }

    /// Remove liquidity from a token pair
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let tx_hash = uniswap_service.remove_liquidity(
    ///     router_address,
    ///     token_a,
    ///     token_b,
    ///     U256::from(1000000),
    ///     U256::from(900000),
    ///     U256::from(800000),
    ///     1698765432
    /// ).await?;
    /// ```
    pub async fn remove_liquidity(
        &self,
        router_address: Address,
        token_a: Address,
        token_b: Address,
        liquidity: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        deadline: u64,
    ) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let wallet_address = self.client.wallet.as_ref().unwrap().address();
        let router = self.router.v2_router(router_address);
        let tx = router.remove_liquidity(
            token_a,
            token_b,
            liquidity,
            amount_a_min,
            amount_b_min,
            wallet_address,
            deadline.into(),
        );
        let pending_tx = tx.send().await.map_err(|e| {
            EvmError::TransactionError(format!("Failed to remove liquidity: {}", e))
        })?;
        Ok(pending_tx.tx_hash())
    }

    /// Remove liquidity with ETH
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let tx_hash = uniswap_service.remove_liquidity_eth(
    ///     router_address,
    ///     token_a,
    ///     U256::from(1000000),
    ///     U256::from(900000),
    ///     U256::from(1000000000000000000u64),
    ///     1698765432
    /// ).await?;
    /// ```
    pub async fn remove_liquidity_eth(
        &self,
        router_address: Address,
        token: Address,
        liquidity: U256,
        amount_token_min: U256,
        amount_eth_min: U256,
        deadline: u64,
    ) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let wallet_address = self.client.wallet.as_ref().unwrap().address();
        let router = self.router.v2_router(router_address);
        let tx = router.remove_liquidity_eth(
            token,
            liquidity,
            amount_token_min,
            amount_eth_min,
            wallet_address,
            deadline.into(),
        );
        let pending_tx = tx.send().await.map_err(|e| {
            EvmError::TransactionError(format!("Failed to remove liquidity with ETH: {}", e))
        })?;
        Ok(pending_tx.tx_hash())
    }

    pub fn event_listener(&self) -> UniswapEventListener {
        UniswapEventListener::new(self.client.clone())
    }

    /// Start monitoring swap events for specified pairs
    ///
    /// # Example
    /// ```
    /// uniswap_service.start_monitoring_swaps(
    ///     vec![pair_address1, pair_address2],
    ///     |swap_event| {
    ///         println!("Swap detected: {:?}", swap_event);
    ///     }
    /// ).await?;
    /// ```
    pub async fn start_monitoring_swaps(
        &self,
        pair_addresses: Vec<Address>,
        on_swap: impl Fn(SwapEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.event_listener()
            .start_swap_listener(pair_addresses, on_swap)
            .await
    }

    /// Start monitoring new pair creation events
    ///
    /// # Example
    /// ```
    /// uniswap_service.start_monitoring_new_pairs(
    ///     factory_address,
    ///     |pair_created| {
    ///         println!("New pair created: {:?}", pair_created);
    ///     }
    /// ).await?;
    /// ```
    pub async fn start_monitoring_new_pairs(
        &self,
        factory_address: Address,
        on_pair_created: impl Fn(PairCreatedEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.event_listener()
            .start_pair_created_listener(vec![factory_address], on_pair_created)
            .await
    }
}

/// Service for interacting with Uniswap V4 protocol
pub struct UniswapV4Service {
    client: Arc<EvmClient>,
}

impl UniswapV4Service {
    /// Create a new UniswapV4Service instance
    pub fn new(client: Arc<EvmClient>) -> Self {
        Self { client }
    }

    fn pool_manager(
        &self,
        manager_address: Address,
    ) -> IPoolManager<Provider<ethers::providers::Http>> {
        IPoolManager::new(manager_address, self.client.provider.clone())
    }

    fn v4_pool(&self, pool_address: Address) -> IPool<Provider<ethers::providers::Http>> {
        IPool::new(pool_address, self.client.provider.clone())
    }

    /// Initialize a new V4 pool
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    ///
    /// let pool_address = v4_service.initialize_pool(
    ///     manager_address,
    ///     currency0,
    ///     currency1,
    ///     3000,
    ///     60,
    ///     hooks_address,
    ///     U256::from(79228162514264337593543950336u128),
    ///     vec![]
    /// ).await?;
    /// ```
    pub async fn initialize_pool(
        &self,
        manager_address: Address,
        currency0: Address,
        currency1: Address,
        fee: u32,
        tick_spacing: i32,
        hooks: Address,
        sqrt_price_x96: U256,
        hook_data: Vec<u8>,
    ) -> Result<Address, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let pool_manager = self.pool_manager(manager_address);
        let tx = pool_manager.initialize(
            currency0,
            currency1,
            fee,
            tick_spacing,
            hooks,
            sqrt_price_x96.into(),
            hook_data.into(),
        );
        let pending_tx = tx.send().await.map_err(|e| {
            EvmError::TransactionError(format!("Failed to initialize V4 pool: {}", e))
        })?;
        let receipt = pending_tx.await.map_err(|e| {
            EvmError::TransactionError(format!("Failed to get transaction receipt: {}", e))
        })?;
        if let Some(log) = receipt.unwrap().logs.first() {
            Ok(log.address)
        } else {
            Err(EvmError::ContractError(
                "Failed to parse pool address from logs".to_string(),
            ))
        }
    }

    /// Execute a swap on V4 pool
    ///
    /// # Example
    /// ```
    /// let (amount0, amount1) = v4_service.swap(
    ///     manager_address,
    ///     currency0,
    ///     currency1,
    ///     3000,
    ///     60,
    ///     hooks_address,
    ///     true,
    ///     1000000,
    ///     U256::from(0),
    ///     vec![]
    /// ).await?;
    /// ```
    pub async fn swap(
        &self,
        manager_address: Address,
        currency0: Address,
        currency1: Address,
        fee: u32,
        tick_spacing: i32,
        hooks: Address,
        zero_for_one: bool,
        amount_specified: i128,
        sqrt_price_limit_x96: U256,
        hook_data: Vec<u8>,
    ) -> Result<(i128, i128), EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let pool_manager = self.pool_manager(manager_address);
        let tx = pool_manager.swap(
            currency0,
            currency1,
            fee,
            tick_spacing,
            hooks,
            zero_for_one,
            amount_specified.into(),
            sqrt_price_limit_x96.into(),
            hook_data.into(),
        );
        let result = tx
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to execute V4 swap: {}", e)))?;
        Ok((result.0.as_i128(), result.1.as_i128()))
    }

    /// Modify position in V4 pool
    ///
    /// # Example
    /// ```
    /// let (amount0, amount1) = v4_service.modify_position(
    ///     manager_address,
    ///     currency0,
    ///     currency1,
    ///     3000,
    ///     60,
    ///     hooks_address,
    ///     -600,
    ///     600,
    ///     1000000,
    ///     vec![]
    /// ).await?;
    /// ```
    pub async fn modify_position(
        &self,
        manager_address: Address,
        currency0: Address,
        currency1: Address,
        fee: u32,
        tick_spacing: i32,
        hooks: Address,
        tick_lower: i32,
        tick_upper: i32,
        liquidity_delta: i128,
        hook_data: Vec<u8>,
    ) -> Result<(i128, i128), EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let pool_manager = self.pool_manager(manager_address);
        let tx = pool_manager.modify_position(
            currency0,
            currency1,
            fee,
            tick_spacing,
            hooks,
            tick_lower,
            tick_upper,
            liquidity_delta.into(),
            hook_data.into(),
        );
        let result = tx
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to modify V4 position: {}", e)))?;
        Ok((result.0.as_i128(), result.1.as_i128()))
    }

    /// Get pool address for given parameters
    ///
    /// # Example
    /// ```
    /// let pool_address = v4_service.get_pool_address(
    ///     manager_address,
    ///     currency0,
    ///     currency1,
    ///     3000,
    ///     60,
    ///     hooks_address
    /// ).await?;
    /// ```
    pub async fn get_pool_address(
        &self,
        manager_address: Address,
        currency0: Address,
        currency1: Address,
        fee: u32,
        tick_spacing: i32,
        hooks: Address,
    ) -> Result<Address, EvmError> {
        let pool_manager = self.pool_manager(manager_address);
        let pool_address = pool_manager
            .get_pool(currency0, currency1, fee, tick_spacing, hooks)
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get pool address: {}", e)))?;
        Ok(pool_address)
    }

    /// Get pool information
    ///
    /// # Example
    /// ```
    /// let pool_info = v4_service.get_pool_info(pool_address).await?;
    /// println!("Pool liquidity: {}", pool_info.liquidity);
    /// println!("Current tick: {}", pool_info.current_tick);
    /// ```
    pub async fn get_pool_info(&self, pool_address: Address) -> Result<V4PoolInfo, EvmError> {
        let pool = self.v4_pool(pool_address);
        let slot0 = pool
            .slot_0()
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get pool slot0: {}", e)))?;
        let liquidity =
            pool.liquidity().call().await.map_err(|e| {
                EvmError::ContractError(format!("Failed to get pool liquidity: {}", e))
            })?;
        let fee_growth_global0_x128 = pool.fee_growth_global_0x128().call().await.map_err(|e| {
            EvmError::ContractError(format!("Failed to get fee growth global0: {}", e))
        })?;

        let fee_growth_global1_x128 = pool.fee_growth_global_1x128().call().await.map_err(|e| {
            EvmError::ContractError(format!("Failed to get fee growth global1: {}", e))
        })?;
        let protocol_fees =
            pool.protocol_fees().call().await.map_err(|e| {
                EvmError::ContractError(format!("Failed to get protocol fees: {}", e))
            })?;
        Ok(V4PoolInfo {
            pool_address,
            currency0: Address::zero(),
            currency1: Address::zero(),
            fee: 0,
            tick_spacing: 0,
            hooks: Address::zero(),
            sqrt_price_x96: slot0.0.into(),
            current_tick: slot0.1,
            liquidity: liquidity.into(),
            fee_growth_global0_x128,
            fee_growth_global1_x128,
            protocol_fees_token0: protocol_fees.0.into(),
            protocol_fees_token1: protocol_fees.1.into(),
        })
    }

    /// Get current pool price
    ///
    /// # Example
    /// ```
    /// let price = v4_service.get_pool_price(pool_address).await?;
    /// println!("Current pool price: {}", price);
    /// ```
    pub async fn get_pool_price(&self, pool_address: Address) -> Result<f64, EvmError> {
        let pool = self.v4_pool(pool_address);
        let slot0 = pool
            .slot_0()
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get pool slot0: {}", e)))?;
        let price = cal_price_from_sqrt_price_x96(slot0.0.into());
        Ok(price)
    }

    /// Donate to a V4 pool
    pub async fn donate(
        &self,
        manager_address: Address,
        currency0: Address,
        currency1: Address,
        fee: u32,
        tick_spacing: i32,
        hooks: Address,
        amount0: U256,
        amount1: U256,
        hook_data: Vec<u8>,
    ) -> Result<(i128, i128), EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let pool_manager = self.pool_manager(manager_address);
        let tx = pool_manager.donate(
            currency0,
            currency1,
            fee,
            tick_spacing,
            hooks,
            amount0,
            amount1,
            hook_data.into(),
        );
        let result = tx
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to donate to V4 pool: {}", e)))?;

        Ok((result.0.as_i128(), result.1.as_i128()))
    }

    /// Settle currency balance
    pub async fn settle(
        &self,
        manager_address: Address,
        currency: Address,
    ) -> Result<U256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let pool_manager = self.pool_manager(manager_address);
        let tx = pool_manager.settle(currency);
        let result = tx
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to settle currency: {}", e)))?;
        Ok(result)
    }

    /// Get currency balance for an account
    pub async fn get_currency_balance(
        &self,
        manager_address: Address,
        account: Address,
        currency: Address,
    ) -> Result<U256, EvmError> {
        let pool_manager = self.pool_manager(manager_address);
        let balance = U256::zero();
        todo!();
        Ok(balance)
    }

    /// Get position information
    ///
    /// # Example
    /// ```
    /// let position_info = v4_service.get_position_info(pool_address, position_key).await?;
    /// println!("Position liquidity: {}", position_info.liquidity);
    /// ```
    pub async fn get_position_info(
        &self,
        pool_address: Address,
        position_key: [u8; 32],
    ) -> Result<V4PositionInfo, EvmError> {
        let pool = self.v4_pool(pool_address);
        let position = pool
            .positions(position_key.into())
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get position info: {}", e)))?;
        Ok(V4PositionInfo {
            position_key: position_key.into(),
            owner: Address::zero(),
            liquidity: position.0.into(),
            fee_growth_inside0_last_x128: position.1,
            fee_growth_inside1_last_x128: position.2,
            tokens_owed0: position.3.into(),
            tokens_owed1: position.4.into(),
        })
    }

    /// Get tick information
    ///
    /// # Example
    /// ```
    /// let tick_info = v4_service.get_tick_info(pool_address, 1200).await?;
    /// println!("Tick liquidity gross: {}", tick_info.liquidity_gross);
    /// ```
    pub async fn get_tick_info(
        &self,
        pool_address: Address,
        tick: i32,
    ) -> Result<TickInfo, EvmError> {
        let pool = self.v4_pool(pool_address);
        let tick_info = pool
            .ticks(tick)
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get tick info: {}", e)))?;
        Ok(TickInfo {
            liquidity_gross: tick_info.0.into(),
            liquidity_net: tick_info.1 as i128,
            fee_growth_outside0_x128: tick_info.2,
            fee_growth_outside1_x128: tick_info.3,
            tick_cumulative_outside: tick_info.4 as i64,
            seconds_per_liquidity_outside_x128: tick_info.5,
            seconds_outside: tick_info.6,
            initialized: tick_info.7,
        })
    }
}

pub struct UniswapConfig;

impl UniswapConfig {
    pub fn v2_router_address(chain: crate::EvmType) -> Result<Address, EvmError> {
        match chain {
            crate::EvmType::Ethereum => Ok(str_to_address(
                crate::global::ethereum::mainnet::dex::uniswap::ROUTER_V2_ADDRESS,
            )
            .unwrap()),
            crate::EvmType::Arb => Ok(str_to_address(
                crate::global::arb::mainnet::dex::uniswap::ROUTER_V2_ADDRESS,
            )
            .unwrap()),
            crate::EvmType::Bsc => Ok(str_to_address(
                crate::global::bsc::mainnet::dex::uniswap::ROUTER_V2_ADDRESS,
            )
            .unwrap()),
            crate::EvmType::Base => Ok(str_to_address(
                crate::global::base::mainnet::dex::uniswap::ROUTER_V2_ADDRESS,
            )
            .unwrap()),
            _ => Err(EvmError::ConfigError(
                "Unsupported chain for Uniswap V2".to_string(),
            )),
        }
    }

    pub fn v2_factory_address(chain: crate::EvmType) -> Result<Address, EvmError> {
        match chain {
            crate::EvmType::Ethereum => {
                Ok(str_to_address("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap())
            }
            crate::EvmType::Arb => {
                Ok(str_to_address("0xc35DADB65012eC5796536bD9864eD8773aBc74C4").unwrap())
            }
            crate::EvmType::Bsc => {
                Ok(str_to_address("0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73").unwrap())
            }
            crate::EvmType::Base => {
                Ok(str_to_address("0x8909Dc15e40173Ff4699343b6eB8132c65e18eC6").unwrap())
            }
            _ => Err(EvmError::ConfigError(
                "Unsupported chain for Uniswap V2".to_string(),
            )),
        }
    }

    pub fn v3_router_address(chain: crate::EvmType) -> Result<Address, EvmError> {
        match chain {
            crate::EvmType::Ethereum => {
                Ok(str_to_address("0xE592427A0AEce92De3Edee1F18E0157C05861564").unwrap())
            }
            crate::EvmType::Arb => {
                Ok(str_to_address("0xE592427A0AEce92De3Edee1F18E0157C05861564").unwrap())
            }
            crate::EvmType::Bsc => {
                Ok(str_to_address("0xB971eF87ede563556b2ED4b1C0b0019111Dd85d2").unwrap())
            }
            crate::EvmType::Base => {
                Ok(str_to_address("0x2626664c2603336E57B271c5C0b26F421741e481").unwrap())
            }
            _ => Err(EvmError::ConfigError(
                "Unsupported chain for Uniswap V3".to_string(),
            )),
        }
    }
}
