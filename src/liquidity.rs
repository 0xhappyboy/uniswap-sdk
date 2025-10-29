use crate::factory::Factory;
use crate::price::Price;
use crate::{EvmClient, EvmError, UniswapConfig};
use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct LiquidityPool {
    price: Arc<Price>,
}
impl LiquidityPool {
    pub fn new(price: Arc<Price>) -> Self {
        Self { price: price }
    }
    async fn get_pair_liquidity_details(
        &self,
        pair_address: Address,
    ) -> Result<LiquidityPoolInfo, EvmError> {
        let (token0, token1, reserve0, reserve1, timestamp) =
            self.price.get_pair_info(pair_address).await?;

        let (token0_symbol, token0_decimals) = self
            .price
            .get_token_info(token0)
            .await
            .unwrap_or((None, 18));
        let (token1_symbol, token1_decimals) = self
            .price
            .get_token_info(token1)
            .await
            .unwrap_or((None, 18));
        let tvl = Price::calculate_tvl(reserve0, reserve1, token0_decimals, token1_decimals);
        let volume_24h = self.price.estimate_24h_volume(pair_address).await;
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPoolInfo {
    pub pair_address: Address,
    pub token0: Address,
    pub token1: Address,
    pub token0_symbol: Option<String>,
    pub token1_symbol: Option<String>,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub reserve0: U256,
    pub reserve1: U256,
    pub tvl: f64,
    pub volume_24h: f64,
    pub fee_tier: Option<u32>,
    pub lp_token_supply: U256,
    pub last_updated: u64,
}

impl LiquidityPoolInfo {}

pub struct LiquidityPoolFinder {
    client: Arc<EvmClient>,
    price: Arc<Price>,
    factory: Arc<Factory>,
}

impl LiquidityPoolFinder {
    pub fn new(client: Arc<EvmClient>) -> Self {
        Self {
            client: client.clone(),
            price: Arc::new(Price::new(client.clone())),
            factory: Arc::new(Factory::new(client)),
        }
    }

    pub async fn find_liquidity_pools(
        &self,
        token_address: Address,
        chain: crate::EvmType,
    ) -> Result<Vec<LiquidityPoolInfo>, EvmError> {
        let mut all_pools = Vec::new();
        let v2_pools = self.find_v2_pools(token_address, chain).await?;
        all_pools.extend(v2_pools);
        let v3_pools = self.find_v3_pools(token_address, chain).await?;
        all_pools.extend(v3_pools);
        all_pools.sort_by(|a, b| b.tvl.partial_cmp(&a.tvl).unwrap());
        Ok(all_pools)
    }

    async fn find_v2_pools(
        &self,
        token_address: Address,
        chain: crate::EvmType,
    ) -> Result<Vec<LiquidityPoolInfo>, EvmError> {
        let factory_address = UniswapConfig::v2_factory_address(chain)?;
        let mut pools = Vec::new();
        let common_tokens = self.get_common_tokens(chain).await;
        for &common_token in &common_tokens {
            if common_token == token_address {
                continue;
            }

            if let Ok(pair_address) = self
                .factory
                .get_pair_address(factory_address, token_address, common_token)
                .await
            {
                if !pair_address.is_zero() {
                    if let Ok(pool_info) = self.get_v2_pool_details(pair_address).await {
                        pools.push(pool_info);
                    }
                }
            }
        }
        Ok(pools)
    }

    async fn find_v3_pools(
        &self,
        token_address: Address,
        chain: crate::EvmType,
    ) -> Result<Vec<LiquidityPoolInfo>, EvmError> {
        let mut pools = Vec::new();
        todo!();
        Ok(pools)
    }

    async fn get_v2_pool_details(
        &self,
        pair_address: Address,
    ) -> Result<LiquidityPoolInfo, EvmError> {
        let (token0, token1, reserve0, reserve1, timestamp) =
            self.price.get_pair_info(pair_address).await?;

        let (token0_symbol, token0_decimals) =
            self.get_token_info(token0).await.unwrap_or((None, 18));
        let (token1_symbol, token1_decimals) =
            self.get_token_info(token1).await.unwrap_or((None, 18));

        let tvl = self.calculate_pool_tvl(reserve0, reserve1, token0_decimals, token1_decimals);

        let volume_24h = self.estimate_pool_volume(pair_address).await;

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

    async fn get_token_info(
        &self,
        token_address: Address,
    ) -> Result<(Option<String>, u8), EvmError> {
        Ok((None, 18))
    }

    fn calculate_pool_tvl(&self, reserve0: U256, reserve1: U256, dec0: u8, dec1: u8) -> f64 {
        let reserve0_adj = reserve0.as_u128() as f64 / 10f64.powi(dec0 as i32);
        let reserve1_adj = reserve1.as_u128() as f64 / 10f64.powi(dec1 as i32);

        reserve0_adj + reserve1_adj
    }

    /// Estimates 24-hour trading volume for a pool
    async fn estimate_pool_volume(&self, pair_address: Address) -> f64 {
        let (reserve0, reserve1, _) = match self.price.get_reserves(pair_address).await {
            Ok(reserves) => reserves,
            Err(_) => return 0.0,
        };

        (reserve0.as_u128() as f64 + reserve1.as_u128() as f64) / 1e18 * 0.05
    }

    pub async fn get_lp_token_supply(pair_address: Address) -> Result<U256, EvmError> {
        todo!();
        Ok(U256::from(1000000))
    }

    async fn get_common_tokens(&self, chain: crate::EvmType) -> Vec<Address> {
        match chain {
            crate::EvmType::Ethereum => vec![
                "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
                    .parse()
                    .unwrap(),
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
                    .parse()
                    .unwrap(),
                "0xdAC17F958D2ee523a2206206994597C13D831ec7"
                    .parse()
                    .unwrap(),
                "0x6B175474E89094C44Da98b954EedeAC495271d0F"
                    .parse()
                    .unwrap(),
            ],
            crate::EvmType::Bsc => vec![
                "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c"
                    .parse()
                    .unwrap(),
                "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56"
                    .parse()
                    .unwrap(),
                "0x55d398326f99059fF775485246999027B3197955"
                    .parse()
                    .unwrap(),
            ],
            _ => vec![],
        }
    }

    /// Finds the liquidity pool with the highest TVL for a token
    ///
    /// # Example
    /// ```rust
    /// use ethers::types::Address;
    /// use std::str::FromStr;
    /// use crate::EvmType;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let finder = create_liquidity_finder();
    /// let token = Address::from_str("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984")?;
    /// if let Some(top_pool) = finder.find_top_liquidity_pool(token, EvmType::Ethereum).await? {
    ///     println!("Top pool: {} with TVL: ${}", top_pool.pair_address, top_pool.tvl);
    /// }
    /// Ok(())
    /// }
    /// ```
    pub async fn find_top_liquidity_pool(
        &self,
        token_address: Address,
        chain: crate::EvmType,
    ) -> Result<Option<LiquidityPoolInfo>, EvmError> {
        let pools = self.find_liquidity_pools(token_address, chain).await?;
        Ok(pools.into_iter().next())
    }

    /// Finds pools with TVL above a specified threshold
    ///
    /// # Example
    /// ```rust
    /// use ethers::types::Address;
    /// use std::str::FromStr;
    /// use crate::EvmType;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let finder = create_liquidity_finder();
    /// let token = Address::from_str("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984")?;
    /// let min_tvl = 1000000.0; // $1M minimum
    /// let high_tvl_pools = finder.find_pools_by_tvl_threshold(token, EvmType::Ethereum, min_tvl).await?;
    /// println!("Found {} pools with TVL > ${}", high_tvl_pools.len(), min_tvl);
    /// Ok(())
    /// }
    /// ```
    pub async fn find_pools_by_tvl_threshold(
        &self,
        token_address: Address,
        chain: crate::EvmType,
        min_tvl: f64,
    ) -> Result<Vec<LiquidityPoolInfo>, EvmError> {
        let pools = self.find_liquidity_pools(token_address, chain).await?;
        let filtered: Vec<LiquidityPoolInfo> = pools
            .into_iter()
            .filter(|pool| pool.tvl >= min_tvl)
            .collect();
        Ok(filtered)
    }

    /// Calculates a health score for a liquidity pool (0-100)
    ///
    /// # Example
    /// ```rust
    /// use ethers::types::Address;
    /// use std::str::FromStr;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let finder = create_liquidity_finder();
    /// let pool_address = Address::from_str("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852")?;
    /// let health_score = finder.get_pool_health_score(pool_address).await?;
    /// println!("Pool health score: {}/100", health_score);
    /// Ok(())
    /// }
    /// ```
    pub async fn get_pool_health_score(&self, pool_address: Address) -> Result<f64, EvmError> {
        let pool_info = self.get_v2_pool_details(pool_address).await?;

        let mut score = 100.0;

        if pool_info.tvl < 1000.0 {
            score -= 30.0;
        } else if pool_info.tvl < 10000.0 {
            score -= 10.0;
        }

        let volume_ratio = pool_info.volume_24h / pool_info.tvl;
        if volume_ratio < 0.01 {
            score -= 20.0;
        }

        let reserve_ratio = if !pool_info.reserve1.is_zero() {
            pool_info.reserve0.as_u128() as f64 / pool_info.reserve1.as_u128() as f64
        } else {
            1.0
        };

        if reserve_ratio < 0.1 || reserve_ratio > 10.0 {
            score -= 15.0;
        }

        Ok(f64::max(score, 0.0))
    }

    /// Provides comprehensive liquidity overview for a token
    ///
    /// # Example
    /// ```rust
    /// use ethers::types::Address;
    /// use std::str::FromStr;
    /// use crate::EvmType;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let finder = create_liquidity_finder();
    /// let token = Address::from_str("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984")?;
    /// let overview = finder.get_token_liquidity_overview(token, EvmType::Ethereum).await?;
    /// println!("Total TVL: ${}", overview.total_tvl);
    /// println!("Total 24h Volume: ${}", overview.total_volume);
    /// println!("Number of pools: {}", overview.pool_count);
    /// Ok(())
    /// }
    /// ```
    pub async fn get_token_liquidity_overview(
        &self,
        token_address: Address,
        chain: crate::EvmType,
    ) -> Result<TokenLiquidityOverview, EvmError> {
        let pools = self.find_liquidity_pools(token_address, chain).await?;
        let total_tvl: f64 = pools.iter().map(|p| p.tvl).sum();
        let total_volume: f64 = pools.iter().map(|p| p.volume_24h).sum();
        let pool_count = pools.len();
        let top_pool = pools.first().cloned();
        Ok(TokenLiquidityOverview {
            token_address,
            total_tvl,
            total_volume,
            pool_count,
            top_pool,
            all_pools: pools,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolComparison {
    pub tvl_ratio: f64,
    pub volume_ratio: f64,
    pub depth_advantage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLiquidityOverview {
    pub token_address: Address,
    pub total_tvl: f64,
    pub total_volume: f64,
    pub pool_count: usize,
    pub top_pool: Option<LiquidityPoolInfo>,
    pub all_pools: Vec<LiquidityPoolInfo>,
}
