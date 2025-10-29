use crate::EvmError;
use crate::abi::IUniswapV2Pair;
use crate::events::UniswapEventManager;
use crate::factory::Factory;
use crate::liquidity::{LiquidityPoolFinder, LiquidityPoolInfo};
use crate::tool::{cal_swap_volume, parse_swap_log};
use crate::types::SwapEvent;
use ethers::abi::Bytes;
use ethers::providers::Middleware;
use ethers::providers::Provider;
use ethers::types::{Address, TransactionRequest, U256};
use evm_sdk::Evm;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Represents price information for a Uniswap V2 pair
#[derive(Debug, Clone)]
pub struct PairPrice {
    pub pair_address: Address,
    pub token0: Address,
    pub token1: Address,
    pub reserve0: U256,
    pub reserve1: U256,
    pub price0: f64,
    pub price1: f64,
}

/// Main price calculation service for Uniswap pairs and tokens
pub struct Price {
    evm: Arc<Evm>,
    factory: Arc<Factory>,
    event_manager: Arc<UniswapEventManager>,
}

impl Price {
    /// Creates a new Price service instance
    pub fn new(evm: Arc<Evm>) -> Self {
        Self {
            evm: evm.clone(),
            factory: Arc::new(Factory::new(evm.clone())),
            event_manager: Arc::new(UniswapEventManager::new(evm.clone())),
        }
    }

    /// Automatically finds price pairs and returns USD and ETH prices for a token
    ///
    /// # Example
    /// ```
    /// let price_service = Price::new(client);
    /// let (usd_price, eth_price) = price_service.get_token_price_auto(
    ///     token_address,
    ///     EvmType::Ethereum
    /// ).await?;
    /// ```
    pub async fn get_token_price_auto(
        &self,
        token_address: Address,
        chain: crate::EvmType,
    ) -> Result<(f64, f64), EvmError> {
        let (usd_pair_v2, eth_pair_v2, v3_pool_usd, v3_pool_eth) =
            self.factory.find_price_pairs(token_address, chain).await?;
        self.get_erc20_price(
            token_address,
            usd_pair_v2,
            eth_pair_v2,
            v3_pool_usd,
            v3_pool_eth,
        )
        .await
    }

    /// Calculates price impact for a swap in a given pair
    ///
    /// # Example
    /// ```
    /// let price_impact = price_service.cal_price_impact(
    ///     pair_address,
    ///     amount_in,
    ///     true  // is_token0_in
    /// ).await?;
    /// ```
    pub async fn cal_price_impact(
        &self,
        pair_address: Address,
        amount_in: U256,
        is_token0_in: bool,
    ) -> Result<f64, EvmError> {
        let (reserve0, reserve1, _) = self.get_reserves(pair_address).await?;
        let (reserve_in, reserve_out) = if is_token0_in {
            (reserve0, reserve1)
        } else {
            (reserve1, reserve0)
        };
        if reserve_in.is_zero() || reserve_out.is_zero() {
            return Ok(0.0);
        }
        let amount_in_with_fee = amount_in * U256::from(997);
        let amount_out = (amount_in_with_fee * reserve_out)
            / (reserve_in * U256::from(1000) + amount_in_with_fee);
        let old_price = reserve_out.as_u128() as f64 / reserve_in.as_u128() as f64;
        let new_reserve_in = reserve_in + amount_in;
        let new_reserve_out = reserve_out - amount_out;
        if new_reserve_out.is_zero() {
            return Ok(100.0);
        }
        let new_price = new_reserve_out.as_u128() as f64 / new_reserve_in.as_u128() as f64;
        let price_impact = ((old_price - new_price) / old_price) * 100.0;
        Ok(price_impact.abs())
    }

    fn v2_pair(&self, pair_address: Address) -> IUniswapV2Pair<Provider<ethers::providers::Http>> {
        IUniswapV2Pair::new(pair_address, self.evm.client.provider.clone())
    }

    /// Gets reserves for a pair address
    pub async fn get_reserves(&self, pair_address: Address) -> Result<(U256, U256, u32), EvmError> {
        let pair = self.v2_pair(pair_address);
        let (reserve0, reserve1, block_timestamp_last) = pair
            .get_reserves()
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get reserves: {}", e)))?;

        Ok((
            U256::from(reserve0),
            U256::from(reserve1),
            block_timestamp_last,
        ))
    }

    /// Gets price from Uniswap V3 pool
    pub async fn get_v3_price(
        &self,
        token_address: Address,
        pool_address: Address,
    ) -> Result<f64, EvmError> {
        let (reserve0, reserve1, _) = self.get_reserves(pool_address).await?;
        let token0 = self.get_token0(pool_address).await?;
        let token1 = self.get_token1(pool_address).await?;
        if token0 == token_address {
            if !reserve1.is_zero() {
                Ok(reserve0.as_u128() as f64 / reserve1.as_u128() as f64)
            } else {
                Ok(0.0)
            }
        } else if token1 == token_address {
            if !reserve0.is_zero() {
                Ok(reserve1.as_u128() as f64 / reserve0.as_u128() as f64)
            } else {
                Ok(0.0)
            }
        } else {
            Err(EvmError::ContractError(
                "Target token not found in V3 pool".to_string(),
            ))
        }
    }

    /// Gets token information (symbol and decimals)
    pub async fn get_token_info(
        &self,
        token_address: Address,
    ) -> Result<(Option<String>, u8), EvmError> {
        todo!();
        Ok((None, 18))
    }

    /// Gets comprehensive current token price information
    ///
    /// # Example
    /// ```
    /// let current_price = price_service.get_token_current_price(
    ///     token_address,
    ///     EvmType::Ethereum
    /// ).await?;
    /// println!("USD Price: {}", current_price.price_usd);
    /// ```
    pub async fn get_token_current_price(
        &self,
        token_address: Address,
        chain: crate::EvmType,
    ) -> Result<CurrentTokenPrice, EvmError> {
        let (usd_pair_v2, eth_pair_v2, v3_pool_usd, v3_pool_eth) =
            self.factory.find_price_pairs(token_address, chain).await?;
        let (price_usd, price_eth) = self
            .get_erc20_price(
                token_address,
                usd_pair_v2,
                eth_pair_v2,
                v3_pool_usd,
                v3_pool_eth,
            )
            .await?;
        let mut liquidity_info = Vec::new();
        if let Some(pair) = usd_pair_v2 {
            if let Ok(liquidity) = self.get_pair_liquidity_details(pair).await {
                liquidity_info.push(liquidity);
            }
        }
        if let Some(pair) = eth_pair_v2 {
            if let Ok(liquidity) = self.get_pair_liquidity_details(pair).await {
                liquidity_info.push(liquidity);
            }
        }
        let price_change = self.cal_recent_price_change(token_address, chain).await;
        Ok(CurrentTokenPrice {
            token_address,
            price_usd,
            price_eth,
            liquidity_pools: liquidity_info,
            price_change_24h: price_change,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source_pairs: vec![usd_pair_v2, eth_pair_v2]
                .into_iter()
                .flatten()
                .collect(),
        })
    }

    /// Gets ERC20 token price from multiple sources
    pub async fn get_erc20_price(
        &self,
        token_address: Address,
        usd_pair_address: Option<Address>,
        eth_pair_address: Option<Address>,
        v3_pool_usd: Option<Address>,
        v3_pool_eth: Option<Address>,
    ) -> Result<(f64, f64), EvmError> {
        let mut usd_price = 0.0;
        let mut eth_price = 0.0;
        if let Some(usd_pair) = usd_pair_address {
            if let Ok(price) = self.get_v2_price(token_address, usd_pair).await {
                usd_price = price;
            }
        }
        if let Some(eth_pair) = eth_pair_address {
            if let Ok(price) = self.get_v2_price(token_address, eth_pair).await {
                eth_price = price;
            }
        }
        if let Some(v3_pool) = v3_pool_usd {
            if let Ok(price) = self.get_v3_price(token_address, v3_pool).await {
                if usd_price > 0.0 {
                    usd_price = (usd_price + price) / 2.0;
                } else {
                    usd_price = price;
                }
            }
        }
        if let Some(v3_pool) = v3_pool_eth {
            if let Ok(price) = self.get_v3_price(token_address, v3_pool).await {
                if eth_price > 0.0 {
                    eth_price = (eth_price + price) / 2.0;
                } else {
                    eth_price = price;
                }
            }
        }
        if usd_price == 0.0 && eth_price == 0.0 {
            return Err(EvmError::ContractError(
                "No valid price data found for token".to_string(),
            ));
        }
        Ok((usd_price, eth_price))
    }

    async fn get_v2_price(
        &self,
        token_address: Address,
        pair_address: Address,
    ) -> Result<f64, EvmError> {
        let pair_price = self.get_pair_price(pair_address).await?;

        if pair_price.token0 == token_address {
            Ok(pair_price.price1)
        } else if pair_price.token1 == token_address {
            Ok(pair_price.price0)
        } else {
            Err(EvmError::ContractError(
                "Target token not found in V2 pair".to_string(),
            ))
        }
    }

    /// Gets complete price information for a pair
    pub async fn get_pair_price(&self, pair_address: Address) -> Result<PairPrice, EvmError> {
        let (token0, token1, reserve0, reserve1, _) = self.get_pair_info(pair_address).await?;
        let price0 = if !reserve1.is_zero() {
            reserve0.as_u128() as f64 / reserve1.as_u128() as f64
        } else {
            0.0
        };
        let price1 = if !reserve0.is_zero() {
            reserve1.as_u128() as f64 / reserve0.as_u128() as f64
        } else {
            0.0
        };
        Ok(PairPrice {
            pair_address,
            token0,
            token1,
            reserve0,
            reserve1,
            price0,
            price1,
        })
    }

    async fn get_pair_liquidity_details(
        &self,
        pair_address: Address,
    ) -> Result<LiquidityPoolInfo, EvmError> {
        let (token0, token1, reserve0, reserve1, timestamp) =
            self.get_pair_info(pair_address).await?;

        let (token0_symbol, token0_decimals) =
            self.get_token_info(token0).await.unwrap_or((None, 18));
        let (token1_symbol, token1_decimals) =
            self.get_token_info(token1).await.unwrap_or((None, 18));
        let tvl = Self::cal_tvl(reserve0, reserve1, token0_decimals, token1_decimals);
        let volume_24h = self.estimate_24h_volume(pair_address).await;
        let lp_token_supply = LiquidityPoolFinder::get_lp_token_supply(pair_address)
            .await
            .unwrap_or(U256::zero());
        Ok(LiquidityPoolInfo {
            pair_address,
            token0,
            token1,
            token0_symbol,
            token1_symbol,
            token0_decimals,
            token1_decimals,
            reserve0,
            reserve1,
            tvl,
            volume_24h,
            fee_tier: None,
            lp_token_supply,
            last_updated: timestamp as u64,
        })
    }

    /// Calculates Total Value Locked for a pair
    pub fn cal_tvl(reserve0: U256, reserve1: U256, dec0: u8, dec1: u8) -> f64 {
        let reserve0_adj = reserve0.as_u128() as f64 / 10f64.powi(dec0 as i32);
        let reserve1_adj = reserve1.as_u128() as f64 / 10f64.powi(dec1 as i32);

        reserve0_adj + reserve1_adj
    }

    /// Estimates 24h volume for a pair
    pub async fn estimate_24h_volume(&self, pair_address: Address) -> f64 {
        let (reserve0, reserve1, _) = match self.get_reserves(pair_address).await {
            Ok(reserves) => reserves,
            Err(_) => return 0.0,
        };
        (reserve0.as_u128() as f64 + reserve1.as_u128() as f64) / 1e18 * 0.1
    }

    /// Gets complete pair information
    pub async fn get_pair_info(
        &self,
        pair_address: Address,
    ) -> Result<(Address, Address, U256, U256, u32), EvmError> {
        let token0 = self.get_token0(pair_address).await?;
        let token1 = self.get_token1(pair_address).await?;
        let (reserve0, reserve1, timestamp) = self.get_reserves(pair_address).await?;

        Ok((token0, token1, reserve0, reserve1, timestamp))
    }

    /// Gets token0 address for a pair
    pub async fn get_token0(&self, pair_address: Address) -> Result<Address, EvmError> {
        let pair = self.v2_pair(pair_address);
        if let Ok(result) = pair
            .method::<_, Address>("token0", ())
            .unwrap()
            .call()
            .await
        {
            return Ok(result);
        }
        self.get_token_low_level(pair_address, "token0").await
    }

    /// Gets token1 address for a pair
    pub async fn get_token1(&self, pair_address: Address) -> Result<Address, EvmError> {
        let pair = self.v2_pair(pair_address);
        if let Ok(result) = pair
            .method::<_, Address>("token1", ())
            .unwrap()
            .call()
            .await
        {
            return Ok(result);
        }
        self.get_token_low_level(pair_address, "token1").await
    }

    async fn cal_recent_price_change(&self, token_address: Address, chain: crate::EvmType) -> f64 {
        0.0
    }

    async fn get_token_low_level(
        &self,
        pair_address: Address,
        method: &str,
    ) -> Result<Address, EvmError> {
        let selector = match method {
            "token0" => "0x0dfe1681",
            "token1" => "0xd21220a7",
            _ => return Err(EvmError::ContractError("Invalid method".to_string())),
        };
        let calldata = hex::decode(&selector[2..])
            .map_err(|e| EvmError::ContractError(format!("Failed to decode selector: {}", e)))?;
        let tx_request = TransactionRequest::new()
            .to(pair_address)
            .data(Bytes::from(calldata));
        let result = self
            .evm
            .client
            .provider
            .as_ref()
            .call(&tx_request.into(), None)
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to call {}: {}", method, e)))?;

        if result.len() >= 32 {
            let mut address_bytes = [0u8; 20];
            address_bytes.copy_from_slice(&result[12..32]);
            Ok(Address::from_slice(&address_bytes))
        } else {
            Err(EvmError::ContractError(format!(
                "Invalid response length for {}",
                method
            )))
        }
    }
}

/// Comprehensive current token price information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentTokenPrice {
    pub token_address: Address,
    pub price_usd: f64,
    pub price_eth: f64,
    pub liquidity_pools: Vec<LiquidityPoolInfo>,
    pub price_change_24h: f64,
    pub last_updated: u64,
    pub source_pairs: Vec<Address>,
}

impl CurrentTokenPrice {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceDataPoint {
    pub block_number: u64,
    pub timestamp: u64,
    pub price_usd: f64,
    pub price_eth: f64,
    pub volume_24h: f64,
    pub liquidity: f64,
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PriceHistoryConfig {
    pub start_block: u64,
    pub end_block: u64,
    pub batch_size: u64,
    pub min_volume_threshold: f64,
    pub include_zero_volume: bool,
}

impl Default for PriceHistoryConfig {
    fn default() -> Self {
        Self {
            start_block: 0,
            end_block: 0,
            batch_size: 1000,
            min_volume_threshold: 0.0,
            include_zero_volume: false,
        }
    }
}

/// Service for retrieving historical price data
pub struct TokenPriceHistory {
    evm: Arc<Evm>,
    config: PriceHistoryConfig,
    factory: Arc<Factory>,
    price: Arc<Price>,
    event_manager: Arc<UniswapEventManager>,
}

impl TokenPriceHistory {
    /// Creates a new TokenPriceHistory service
    pub fn new(evm: Arc<Evm>, config: PriceHistoryConfig) -> Self {
        Self {
            evm: evm.clone(),
            config,
            price: Arc::new(Price::new(evm.clone())),
            factory: Arc::new(Factory::new(evm.clone())),
            event_manager: Arc::new(UniswapEventManager::new(evm.clone())),
        }
    }

    /// Creates a TokenPriceHistory service with default configuration
    pub fn with_default_config(evm: Arc<Evm>) -> Self {
        let config = PriceHistoryConfig::default();
        Self::new(evm, config)
    }

    /// Gets recent prices for a token
    ///
    /// # Example
    /// ```
    /// let history_service = TokenPriceHistory::with_default_config(client);
    /// let recent_prices = history_service.get_recent_prices(
    ///     token_address,
    ///     EvmType::ETHEREUM_MAINNET,
    ///     1000  // last 1000 blocks
    /// ).await?;
    /// ```
    pub async fn get_recent_prices(
        &self,
        token_address: Address,
        chain: crate::EvmType,
        blocks_back: u64,
    ) -> Result<Vec<PriceDataPoint>, EvmError> {
        let current_block = self
            .evm
            .client
            .provider
            .get_block_number()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get current block: {}", e)))?;
        let start_block = current_block.as_u64().saturating_sub(blocks_back);
        let mut config = self.config.clone();
        config.start_block = start_block;
        config.end_block = current_block.as_u64();
        let history_getter = TokenPriceHistory::new(self.evm.clone(), config);
        history_getter
            .get_full_price_history(token_address, chain)
            .await
    }

    /// Gets full price history for a token
    pub async fn get_full_price_history(
        &self,
        token_address: Address,
        chain: crate::EvmType,
    ) -> Result<Vec<PriceDataPoint>, EvmError> {
        let (usd_pair_v2, eth_pair_v2, v3_pool_usd, v3_pool_eth) =
            self.factory.find_price_pairs(token_address, chain).await?;
        let mut all_pair_addresses = Vec::new();
        if let Some(addr) = usd_pair_v2 {
            all_pair_addresses.push(addr);
        }
        if let Some(addr) = eth_pair_v2 {
            all_pair_addresses.push(addr);
        }
        let price_history = self
            .get_price_from_events(
                token_address,
                &all_pair_addresses,
                usd_pair_v2,
                eth_pair_v2,
                v3_pool_usd,
                v3_pool_eth,
            )
            .await?;
        Ok(price_history)
    }

    async fn get_price_from_events(
        &self,
        token_address: Address,
        pair_addresses: &[Address],
        usd_pair_v2: Option<Address>,
        eth_pair_v2: Option<Address>,
        v3_pool_usd: Option<Address>,
        v3_pool_eth: Option<Address>,
    ) -> Result<Vec<PriceDataPoint>, EvmError> {
        let mut price_points = Vec::new();
        let mut current_block = self.config.start_block;
        let block_cache = Arc::new(Mutex::new(HashMap::new()));
        while current_block <= self.config.end_block {
            let to_block = std::cmp::min(
                current_block + self.config.batch_size,
                self.config.end_block,
            );
            let swap_events = self
                .get_swap_events_in_range(pair_addresses, current_block, to_block)
                .await?;
            for event in swap_events {
                if let Some(price_point) = self
                    .process_swap_event(
                        &event,
                        token_address,
                        usd_pair_v2,
                        eth_pair_v2,
                        v3_pool_usd,
                        v3_pool_eth,
                        block_cache.clone(),
                    )
                    .await?
                {
                    price_points.push(price_point);
                }
            }
            current_block = to_block + 1;
        }
        price_points.sort_by(|a, b| a.block_number.cmp(&b.block_number));
        Ok(price_points)
    }

    async fn get_swap_events_in_range(
        &self,
        pair_addresses: &[Address],
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<SwapEvent>, EvmError> {
        let logs = self
            .event_manager
            .get_historical_events(pair_addresses.to_vec(), "Swap", from_block, to_block)
            .await?;

        let mut swap_events = Vec::new();
        for log in logs {
            if let Ok(swap_event) = parse_swap_log(&log) {
                swap_events.push(swap_event);
            }
        }

        Ok(swap_events)
    }

    async fn process_swap_event(
        &self,
        event: &SwapEvent,
        token_address: Address,
        usd_pair_v2: Option<Address>,
        eth_pair_v2: Option<Address>,
        v3_pool_usd: Option<Address>,
        v3_pool_eth: Option<Address>,
        block_cache: Arc<Mutex<HashMap<u64, u64>>>,
    ) -> Result<Option<PriceDataPoint>, EvmError> {
        let timestamp = self
            .get_block_timestamp(event.block_number, block_cache)
            .await?;

        let (price_usd, price_eth) = self
            .price
            .get_erc20_price(
                token_address,
                usd_pair_v2,
                eth_pair_v2,
                v3_pool_usd,
                v3_pool_eth,
            )
            .await
            .unwrap_or((0.0, 0.0));

        let volume = self.cal_event_volume(event, price_usd, price_eth);

        if !self.config.include_zero_volume && volume < self.config.min_volume_threshold {
            return Ok(None);
        }

        let liquidity = self
            .get_pair_liquidity(event.pair_address)
            .await
            .unwrap_or(0.0);

        let price_point = PriceDataPoint {
            block_number: event.block_number,
            timestamp,
            price_usd,
            price_eth,
            volume_24h: volume,
            liquidity,
            transaction_hash: Some(format!("{:?}", event.transaction_hash)),
        };

        Ok(Some(price_point))
    }

    async fn get_block_timestamp(
        &self,
        block_number: u64,
        block_cache: Arc<Mutex<HashMap<u64, u64>>>,
    ) -> Result<u64, EvmError> {
        {
            let cache = block_cache.lock().await;
            if let Some(&timestamp) = cache.get(&block_number) {
                return Ok(timestamp);
            }
        }

        let block = self
            .evm
            .client
            .provider
            .get_block(block_number)
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get block: {}", e)))?;

        let timestamp = if let Some(block) = block {
            block.timestamp.as_u64()
        } else {
            0
        };

        {
            let mut cache = block_cache.lock().await;
            cache.insert(block_number, timestamp);
        }

        Ok(timestamp)
    }

    fn cal_event_volume(&self, event: &SwapEvent, token0_price: f64, token1_price: f64) -> f64 {
        cal_swap_volume(event, Some(token0_price), Some(token1_price))
    }

    async fn get_pair_liquidity(&self, pair_address: Address) -> Result<f64, EvmError> {
        let (reserve0, reserve1, _) = self.price.get_reserves(pair_address).await?;

        let liquidity = (reserve0.as_u128() as f64) * (reserve1.as_u128() as f64).sqrt();

        Ok(liquidity)
    }

    /// Gets comprehensive price statistics for a token
    ///
    /// # Example
    /// ```
    /// let stats = history_service.get_price_statistics(
    ///     token_address,
    ///     EvmType::Ethereum
    /// ).await?;
    /// println!("24h price change: {}%", stats.price_change_24h_usd);
    /// ```
    pub async fn get_price_statistics(
        &self,
        token_address: Address,
        chain: crate::EvmType,
    ) -> Result<PriceStatistics, EvmError> {
        let price_history = self.get_full_price_history(token_address, chain).await?;

        if price_history.is_empty() {
            return Err(EvmError::ContractError("No price data found".to_string()));
        }

        let prices_usd: Vec<f64> = price_history.iter().map(|p| p.price_usd).collect();
        let prices_eth: Vec<f64> = price_history.iter().map(|p| p.price_eth).collect();
        let volumes: Vec<f64> = price_history.iter().map(|p| p.volume_24h).collect();

        Ok(PriceStatistics {
            token_address,
            total_data_points: price_history.len(),
            current_price_usd: prices_usd.last().copied().unwrap_or(0.0),
            current_price_eth: prices_eth.last().copied().unwrap_or(0.0),
            price_change_24h_usd: self.cal_price_change(&prices_usd, 24),
            price_change_24h_eth: self.cal_price_change(&prices_eth, 24),
            all_time_high_usd: *prices_usd
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(&0.0),
            all_time_low_usd: *prices_usd
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(&0.0),
            average_volume_24h: volumes.iter().sum::<f64>() / volumes.len() as f64,
            max_volume_24h: *volumes
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(&0.0),
            volatility_usd: self.cal_volatility(&prices_usd),
            volatility_eth: self.cal_volatility(&prices_eth),
        })
    }

    fn cal_price_change(&self, prices: &[f64], hours_back: usize) -> f64 {
        if prices.len() <= hours_back {
            return 0.0;
        }

        let current_price = prices.last().unwrap();
        let previous_price = prices[prices.len() - hours_back - 1];

        if previous_price == 0.0 {
            return 0.0;
        }

        ((current_price - previous_price) / previous_price) * 100.0
    }

    fn cal_volatility(&self, prices: &[f64]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }

        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices
            .iter()
            .map(|&price| (price - mean).powi(2))
            .sum::<f64>()
            / prices.len() as f64;

        variance.sqrt()
    }

    /// Exports price history to CSV file
    ///
    /// # Example
    /// ```
    /// history_service.export_to_csv(
    ///     token_address,
    ///     EvmType::Ethereum,
    ///     "price_history.csv"
    /// ).await?;
    /// ```
    pub async fn export_to_csv(
        &self,
        token_address: Address,
        chain: crate::EvmType,
        filename: &str,
    ) -> Result<(), EvmError> {
        let price_history = self.get_full_price_history(token_address, chain).await?;
        let mut wtr = csv::Writer::from_path(filename)
            .map_err(|e| EvmError::IOError(format!("Failed to create CSV file: {}", e)))?;
        wtr.write_record(&[
            "block_number",
            "timestamp",
            "price_usd",
            "price_eth",
            "volume_24h",
            "liquidity",
            "transaction_hash",
        ])
        .map_err(|e| EvmError::IOError(format!("Failed to write CSV header: {}", e)))?;
        for point in price_history {
            wtr.write_record(&[
                point.block_number.to_string(),
                point.timestamp.to_string(),
                point.price_usd.to_string(),
                point.price_eth.to_string(),
                point.volume_24h.to_string(),
                point.liquidity.to_string(),
                point.transaction_hash.unwrap_or_default(),
            ])
            .map_err(|e| EvmError::IOError(format!("Failed to write CSV record: {}", e)))?;
        }
        wtr.flush()
            .map_err(|e| EvmError::IOError(format!("Failed to flush CSV file: {}", e)))?;
        Ok(())
    }

    /// Gets price data within a specific time range
    pub async fn get_price_in_time_range(
        &self,
        token_address: Address,
        chain: crate::EvmType,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> Result<Vec<PriceDataPoint>, EvmError> {
        let full_history = self.get_full_price_history(token_address, chain).await?;

        let filtered_history: Vec<PriceDataPoint> = full_history
            .into_iter()
            .filter(|point| point.timestamp >= start_timestamp && point.timestamp <= end_timestamp)
            .collect();

        Ok(filtered_history)
    }
}

/// Comprehensive price statistics for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceStatistics {
    pub token_address: Address,
    pub total_data_points: usize,
    pub current_price_usd: f64,
    pub current_price_eth: f64,
    pub price_change_24h_usd: f64,
    pub price_change_24h_eth: f64,
    pub all_time_high_usd: f64,
    pub all_time_low_usd: f64,
    pub average_volume_24h: f64,
    pub max_volume_24h: f64,
    pub volatility_usd: f64,
    pub volatility_eth: f64,
}
