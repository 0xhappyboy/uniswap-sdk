use crate::farm::FarmService;
use crate::price::Price;
use crate::types::{FarmPerformance, FarmPool};
use crate::{Evm, EvmError};
use ethers::types::Address;
use ethers::types::U256;
use evm_client::EvmType;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc};

#[derive(Debug, Clone)]
pub struct LiquidityChangeEvent {
    pub pool_address: Address,
    pub token0_delta: i128,
    pub token1_delta: i128,
    pub new_total_liquidity: f64,
    pub block_number: u64,
    pub transaction_hash: String,
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub token: Address,
    pub profit_estimate: f64,
    pub path: Vec<ArbitrageStep>,
    pub confidence: f64,
    pub expiration_block: u64,
}

#[derive(Debug, Clone)]
pub struct ArbitrageStep {
    pub from_token: Address,
    pub to_token: Address,
    pub exchange: String,
    pub expected_rate: f64,
}

#[derive(Debug, Clone)]
pub struct PoolHealthScore {
    pub pool_address: Address,
    pub overall_score: f64,
    pub liquidity_score: f64,
    pub volume_score: f64,
    pub concentration_score: f64,
    pub stability_score: f64,
    pub recommendations: Vec<String>,
}

/// Analytics engine for DeFi protocol analysis including liquidity monitoring,
/// arbitrage detection, and pool health assessment.
pub struct Analyze {
    evm: Arc<Evm>,
}

impl Analyze {
    /// Creates a new Analyze instance.
    pub fn new(evm: Arc<Evm>) -> Self {
        Self { evm: evm }
    }

    /// Monitors liquidity changes for specified pools and returns a receiver for liquidity events.
    ///
    /// # Example
    /// ```
    /// use analytics::Analyze;
    /// use ethers::types::Address;
    /// use std::sync::Arc;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Arc::new(Evm::new(EvmType::Ethereum).await?);
    /// let engine = Analyze::new(client);
    /// let pools = vec![
    ///     "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852".parse()? // USDT-WETH
    /// ];
    /// let mut receiver = engine.monitor_liquidity_changes(pools).await?;
    ///
    /// while let Some(event) = receiver.recv().await {
    ///     println!("Liquidity change: {:?}", event);
    /// }
    /// Ok(())
    /// }
    /// ```
    pub async fn monitor_liquidity_changes(
        &self,
        pools: Vec<Address>,
    ) -> Result<mpsc::Receiver<LiquidityChangeEvent>, EvmError> {
        let (tx, rx) = mpsc::channel(100);
        let evm = self.evm.clone();
        tokio::spawn(async move {
            let mut last_block = match evm.get_block_number().await {
                Ok(block) => block,
                Err(e) => {
                    eprintln!("Failed to get block number: {}", e);
                    return;
                }
            };
            let mut interval = tokio::time::interval(Duration::from_secs(12));
            loop {
                interval.tick().await;
                match evm.get_block_number().await {
                    Ok(current_block) => {
                        if current_block > last_block {
                            let event_manager =
                                crate::events::UniswapEventManager::new(evm.clone());
                            if let Err(e) = Self::process_liquidity_events(
                                evm.clone(),
                                &event_manager,
                                &pools,
                                &tx,
                                last_block + 1,
                                current_block,
                            )
                            .await
                            {
                                eprintln!("Error processing liquidity events: {}", e);
                            }
                            last_block = current_block;
                        }
                    }
                    Err(e) => eprintln!("Failed to get block number: {}", e),
                }
            }
        });
        Ok(rx)
    }

    async fn process_liquidity_events(
        evm: Arc<Evm>,
        event_manager: &crate::events::UniswapEventManager,
        pools: &[Address],
        tx: &mpsc::Sender<LiquidityChangeEvent>,
        from_block: u64,
        to_block: u64,
    ) -> Result<(), EvmError> {
        let mint_logs = event_manager
            .get_historical_events(pools.to_vec(), "Mint", from_block, to_block)
            .await?;
        let burn_logs = event_manager
            .get_historical_events(pools.to_vec(), "Burn", from_block, to_block)
            .await?;
        for log in mint_logs {
            if let Ok(event) = Self::parse_mint_to_liquidity_event(evm.clone(), &log).await {
                let _ = tx.send(event).await;
            }
        }
        for log in burn_logs {
            if let Ok(event) = Self::parse_burn_to_liquidity_event(evm.clone(), &log).await {
                let _ = tx.send(event).await;
            }
        }
        Ok(())
    }

    async fn parse_mint_to_liquidity_event(
        evm: Arc<Evm>,
        log: &ethers::types::Log,
    ) -> Result<LiquidityChangeEvent, EvmError> {
        let pool_address = log.address;
        let price_service = Price::new(evm.clone());
        let (_, _, reserve0, reserve1, _) = price_service.get_pair_info(pool_address).await?;
        let (token0_delta, token1_delta) = if log.data.len() >= 64 {
            let amount0 = U256::from_big_endian(&log.data[0..32]);
            let amount1 = U256::from_big_endian(&log.data[32..64]);
            (amount0.as_u128() as i128, amount1.as_u128() as i128)
        } else {
            (0, 0)
        };
        let total_liquidity = reserve0.as_u128() as f64 + reserve1.as_u128() as f64;
        Ok(LiquidityChangeEvent {
            pool_address,
            token0_delta,
            token1_delta,
            new_total_liquidity: total_liquidity,
            block_number: log.block_number.unwrap().as_u64(),
            transaction_hash: format!("{:?}", log.transaction_hash.unwrap()),
        })
    }

    async fn parse_burn_to_liquidity_event(
        evm: Arc<Evm>,
        log: &ethers::types::Log,
    ) -> Result<LiquidityChangeEvent, EvmError> {
        let pool_address = log.address;
        let price_service = crate::price::Price::new(evm.clone());
        let (_, _, reserve0, reserve1, _) = price_service.get_pair_info(pool_address).await?;
        let (token0_delta, token1_delta) = if log.data.len() >= 64 {
            let amount0 = U256::from_big_endian(&log.data[0..32]);
            let amount1 = U256::from_big_endian(&log.data[32..64]);
            (-(amount0.as_u128() as i128), -(amount1.as_u128() as i128))
        } else {
            (0, 0)
        };
        let total_liquidity = reserve0.as_u128() as f64 + reserve1.as_u128() as f64;
        Ok(LiquidityChangeEvent {
            pool_address,
            token0_delta,
            token1_delta,
            new_total_liquidity: total_liquidity,
            block_number: log.block_number.unwrap().as_u64(),
            transaction_hash: format!("{:?}", log.transaction_hash.unwrap()),
        })
    }

    /// Detects arbitrage opportunities across specified tokens.
    ///
    /// # Example
    /// ```
    /// use analytics::Analyze;
    /// use ethers::types::Address;
    /// use std::sync::Arc;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Arc::new(Evm::new(EvmType::Ethereum).await?);
    /// let engine = Analyze::new(client);
    /// let tokens = vec![
    ///     "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()? // WETH
    /// ];
    /// let opportunities = engine.detect_arbitrage_opportunities(tokens).await?;
    ///
    /// for opportunity in opportunities {
    ///     if opportunity.profit_estimate > 1.0 {
    ///         println!("Profitable arbitrage: {}%", opportunity.profit_estimate);
    ///     }
    /// }
    /// Ok(())
    /// }
    /// ```
    pub async fn detect_arbitrage_opportunities(
        &self,
        tokens: Vec<Address>,
    ) -> Result<Vec<ArbitrageOpportunity>, EvmError> {
        let mut opportunities = Vec::new();
        for &token in &tokens {
            if let Ok(opportunity) = self.analyze_arbitrage_for_token(token).await {
                opportunities.push(opportunity);
            }
        }
        opportunities.sort_by(|a, b| b.profit_estimate.partial_cmp(&a.profit_estimate).unwrap());
        Ok(opportunities)
    }

    /// Calculates a comprehensive health score for a liquidity pool.
    ///
    /// # Example
    /// ```
    /// use analytics::Analyze;
    /// use ethers::types::Address;
    /// use std::sync::Arc;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Arc::new(Evm::new(EvmType::Ethereum).await?);
    /// let engine = Analyze::new(client);
    /// let pool_address = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852".parse()?; // USDT-WETH
    /// let health_score = engine.cal_pool_health_score(pool_address).await?;
    ///
    /// println!("Pool health: {:.1}/100", health_score.overall_score);
    /// for recommendation in &health_score.recommendations {
    ///     println!("Recommendation: {}", recommendation);
    /// }
    /// Ok(())
    /// }
    /// ```
    pub async fn cal_pool_health_score(
        &self,
        pool_address: Address,
    ) -> Result<PoolHealthScore, EvmError> {
        let liquidity_metrics = self.analyze_liquidity_metrics(pool_address).await?;
        let volume_metrics = self.analyze_volume_metrics(pool_address).await?;
        let concentration_metrics = self.analyze_concentration_metrics(pool_address).await?;
        let stability_metrics = self.analyze_stability_metrics(pool_address).await?;
        let overall_score = (liquidity_metrics.score * 0.3
            + volume_metrics.score * 0.3
            + concentration_metrics.score * 0.2
            + stability_metrics.score * 0.2);
        Ok(PoolHealthScore {
            pool_address,
            overall_score,
            liquidity_score: liquidity_metrics.score,
            volume_score: volume_metrics.score,
            concentration_score: concentration_metrics.score,
            stability_score: stability_metrics.score,
            recommendations: self.generate_health_recommendations(
                &liquidity_metrics,
                &volume_metrics,
                &concentration_metrics,
                &stability_metrics,
            ),
        })
    }

    /// Discovers trading pairs from a factory with minimum liquidity requirement.
    ///
    /// # Example
    /// ```
    /// use analytics::Analyze;
    /// use ethers::types::Address;
    /// use std::sync::Arc;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Arc::new(Evm::new(EvmType::Ethereum).await?);
    /// let engine = Analyze::new(client);
    /// let factory = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".parse()?; // Uniswap V2
    /// let min_liquidity = 10000.0; // $10,000 minimum
    /// let pairs = engine.discover_trading_pairs(factory, min_liquidity).await?;
    ///
    /// for pair in pairs.iter().take(10) {
    ///     println!("Pair: {:?} - {:?} (Liquidity: ${:.2})",
    ///              pair.token0, pair.token1, pair.liquidity);
    /// }
    /// Ok(())
    /// }
    /// ```
    pub async fn discover_trading_pairs(
        &self,
        factory_address: Address,
        min_liquidity: f64,
    ) -> Result<Vec<DiscoveredPair>, EvmError> {
        let factory = crate::factory::Factory::new(self.evm.clone());
        let mut discovered_pairs = Vec::new();
        let base_tokens = self.get_base_tokens().await;
        for i in 0..base_tokens.len() {
            for j in (i + 1)..base_tokens.len() {
                let token_a = base_tokens[i];
                let token_b = base_tokens[j];
                if let Ok(pair_address) = factory
                    .get_pair_address(factory_address, token_a, token_b)
                    .await
                {
                    if !pair_address.is_zero() {
                        if let Ok(pair_info) =
                            self.get_pair_details(pair_address, token_a, token_b).await
                        {
                            if pair_info.liquidity >= min_liquidity {
                                discovered_pairs.push(pair_info);
                            }
                        }
                    }
                }
            }
        }
        discovered_pairs.sort_by(|a, b| b.liquidity.partial_cmp(&a.liquidity).unwrap());
        Ok(discovered_pairs)
    }

    async fn get_pair_details(
        &self,
        pair_address: Address,
        token0: Address,
        token1: Address,
    ) -> Result<DiscoveredPair, EvmError> {
        let price_service = crate::price::Price::new(self.evm.clone());
        let (reserve0, reserve1, _) = price_service.get_reserves(pair_address).await?;
        let liquidity = reserve0.as_u128() as f64 + reserve1.as_u128() as f64;
        let creation_block = self
            .get_pair_creation_block(pair_address)
            .await
            .unwrap_or(0);
        Ok(DiscoveredPair {
            token0,
            token1,
            pair_address,
            liquidity,
            creation_block,
        })
    }

    async fn get_base_tokens(&self) -> Vec<Address> {
        vec![
            // WETH
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
                .parse()
                .unwrap(),
            // USDC
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
                .parse()
                .unwrap(),
            // USDT
            "0xdAC17F958D2ee523a2206206994597C13D831ec7"
                .parse()
                .unwrap(),
            // DAI
            "0x6B175474E89094C44Da98b954EedeAC495271d0F"
                .parse()
                .unwrap(),
            // WBTC
            "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"
                .parse()
                .unwrap(),
        ]
    }

    async fn get_pair_creation_block(&self, pair_address: Address) -> Result<u64, EvmError> {
        let event_manager = crate::events::UniswapEventManager::new(self.evm.clone());
        let logs = event_manager
            .get_historical_events(vec![pair_address], "Mint", 0, u64::MAX)
            .await?;
        if let Some(first_log) = logs.first() {
            Ok(first_log.block_number.unwrap().as_u64())
        } else {
            Ok(0)
        }
    }

    /// Analyzes fee efficiency and calculates LP APR for a pool.
    ///
    /// # Example
    /// ```
    /// use analytics::Analyze;
    /// use ethers::types::Address;
    /// use std::sync::Arc;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Arc::new(Evm::new(EvmType::Ethereum).await?);
    /// let engine = Analyze::new(client);
    /// let pool_address = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852".parse()?; // USDT-WETH
    /// let report = engine.analyze_fee_efficiency(pool_address).await?;
    ///
    /// println!("Fee Efficiency Report:");
    /// println!("24h Fees: ${:.2}", report.fee_earnings_24h);
    /// println!("LP APR: {:.2}%", report.lp_apr);
    /// println!("Efficiency Score: {:.1}/100 ({})",
    ///          report.efficiency_score, report.comparison);
    /// Ok(())
    /// }
    /// ```
    pub async fn analyze_fee_efficiency(
        &self,
        pool_address: Address,
    ) -> Result<FeeEfficiencyReport, EvmError> {
        let price_service = crate::price::Price::new(self.evm.clone());
        let volume_24h = self.cal_24h_volume(pool_address).await?;
        let (reserve0, reserve1, _) = price_service.get_reserves(pool_address).await?;
        let total_liquidity = reserve0.as_u128() as f64 + reserve1.as_u128() as f64;
        let fee_earnings_24h = volume_24h * 0.003;
        let lp_apr = if total_liquidity > 0.0 {
            (fee_earnings_24h * 365.0) / total_liquidity * 100.0
        } else {
            0.0
        };
        let efficiency_score = self.cal_efficiency_score(lp_apr, volume_24h, total_liquidity);
        let comparison = Self::get_comparison_rating(efficiency_score);
        Ok(FeeEfficiencyReport {
            pool_address,
            fee_earnings_24h,
            lp_apr,
            efficiency_score,
            comparison,
        })
    }

    async fn cal_24h_volume(&self, pool_address: Address) -> Result<f64, EvmError> {
        let current_block = self
            .evm
            .get_block_number()
            .await
            .map_err(|e| EvmError::Error(format!("{:?}", e)))
            .unwrap();
        let blocks_per_day = 7200u64;
        let from_block = if current_block > blocks_per_day {
            current_block - blocks_per_day
        } else {
            0
        };
        let event_manager = crate::events::UniswapEventManager::new(self.evm.clone());
        let logs = event_manager
            .get_historical_events(vec![pool_address], "Swap", from_block, current_block)
            .await?;
        let mut total_volume = 0.0;
        for log in logs {
            if let Ok(swap_event) = crate::tool::parse_swap_log(&log) {
                let volume = swap_event
                    .amount0_in
                    .as_u128()
                    .max(swap_event.amount1_in.as_u128()) as f64;
                total_volume += volume;
            }
        }
        Ok(total_volume / 1e18)
    }

    fn cal_efficiency_score(&self, apr: f64, volume: f64, liquidity: f64) -> f64 {
        if liquidity == 0.0 {
            return 0.0;
        }
        let volume_ratio = volume / liquidity;
        let apr_score = (apr.min(100.0) / 100.0) * 50.0;
        let volume_score = (volume_ratio.min(1.0)) * 50.0;
        apr_score + volume_score
    }

    fn get_comparison_rating(score: f64) -> String {
        match score {
            s if s >= 80.0 => "Excellent".to_string(),
            s if s >= 60.0 => "Good".to_string(),
            s if s >= 40.0 => "Average".to_string(),
            s if s >= 20.0 => "Below Average".to_string(),
            _ => "Poor".to_string(),
        }
    }

    async fn analyze_arbitrage_for_token(
        &self,
        token: Address,
    ) -> Result<ArbitrageOpportunity, EvmError> {
        let liquidity_finder = crate::liquidity::LiquidityPoolFinder::new(self.evm.clone());
        let pools = liquidity_finder
            .find_liquidity_pools(token, EvmType::ETHEREUM_MAINNET)
            .await?;
        if pools.is_empty() {
            return Ok(ArbitrageOpportunity {
                token,
                profit_estimate: 0.0,
                path: vec![],
                confidence: 0.0,
                expiration_block: self
                    .evm
                    .get_block_number()
                    .await
                    .map_err(|e| EvmError::Error(format!("{:?}", e)))
                    .unwrap()
                    + 5,
            });
        }
        let triangular_opportunity = self.analyze_triangular_arbitrage(token, &pools).await?;
        let cross_dex_opportunity = self.analyze_cross_dex_arbitrage(token, &pools).await?;
        if triangular_opportunity.profit_estimate > cross_dex_opportunity.profit_estimate {
            Ok(triangular_opportunity)
        } else {
            Ok(cross_dex_opportunity)
        }
    }

    async fn analyze_triangular_arbitrage(
        &self,
        token: Address,
        pools: &[crate::liquidity::LiquidityPoolInfo],
    ) -> Result<ArbitrageOpportunity, EvmError> {
        let mut best_profit = 0.0;
        let mut best_path = Vec::new();
        // Finding the Triangular Arbitrage Path: TokenA -> TokenB -> TokenC -> TokenA
        for pool1 in pools {
            let intermediate_token = if pool1.token0 == token {
                pool1.token1
            } else {
                pool1.token0
            };
            for pool2 in pools {
                if pool2.pair_address == pool1.pair_address {
                    continue;
                }
                let final_token = if pool2.token0 == intermediate_token {
                    pool2.token1
                } else if pool2.token1 == intermediate_token {
                    pool2.token0
                } else {
                    continue;
                };
                for pool3 in pools {
                    if (pool3.token0 == final_token && pool3.token1 == token)
                        || (pool3.token1 == final_token && pool3.token0 == token)
                    {
                        let profit = self
                            .cal_triangular_profit(&[
                                (pool1, token, intermediate_token),
                                (pool2, intermediate_token, final_token),
                                (pool3, final_token, token),
                            ])
                            .await?;

                        if profit > best_profit {
                            best_profit = profit;
                            best_path = vec![
                                ArbitrageStep {
                                    from_token: token,
                                    to_token: intermediate_token,
                                    exchange: "UniswapV2".to_string(),
                                    expected_rate: self.cal_exchange_rate(
                                        pool1,
                                        token,
                                        intermediate_token,
                                    ),
                                },
                                ArbitrageStep {
                                    from_token: intermediate_token,
                                    to_token: final_token,
                                    exchange: "UniswapV2".to_string(),
                                    expected_rate: self.cal_exchange_rate(
                                        pool2,
                                        intermediate_token,
                                        final_token,
                                    ),
                                },
                                ArbitrageStep {
                                    from_token: final_token,
                                    to_token: token,
                                    exchange: "UniswapV2".to_string(),
                                    expected_rate: self.cal_exchange_rate(
                                        pool3,
                                        final_token,
                                        token,
                                    ),
                                },
                            ];
                        }
                    }
                }
            }
        }
        Ok(ArbitrageOpportunity {
            token,
            profit_estimate: best_profit,
            path: best_path,
            confidence: if best_profit > 0.0 { 0.8 } else { 0.0 },
            expiration_block: self
                .evm
                .get_block_number()
                .await
                .map_err(|e| EvmError::Error(format!("{:?}", e)))
                .unwrap()
                + 3,
        })
    }

    async fn analyze_cross_dex_arbitrage(
        &self,
        token: Address,
        pools: &[crate::liquidity::LiquidityPoolInfo],
    ) -> Result<ArbitrageOpportunity, EvmError> {
        if pools.len() < 2 {
            return Ok(ArbitrageOpportunity {
                token,
                profit_estimate: 0.0,
                path: vec![],
                confidence: 0.0,
                expiration_block: self
                    .evm
                    .get_block_number()
                    .await
                    .map_err(|e| EvmError::Error(format!("{:?}", e)))
                    .unwrap()
                    + 5,
            });
        }
        let mut best_profit = 0.0;
        let mut best_pools = (None, None);
        for i in 0..pools.len() {
            for j in (i + 1)..pools.len() {
                let pool1 = &pools[i];
                let pool2 = &pools[j];
                let price1 = self.get_token_price_from_pool(pool1, token).await?;
                let price2 = self.get_token_price_from_pool(pool2, token).await?;
                let price_diff = (price1 - price2).abs() / price1.max(price2);
                if price_diff > best_profit {
                    best_profit = price_diff * 100.0;
                    best_pools = (Some(pool1), Some(pool2));
                }
            }
        }

        if let (Some(pool1), Some(pool2)) = best_pools {
            let (buy_pool, sell_pool) = if self.get_token_price_from_pool(pool1, token).await?
                < self.get_token_price_from_pool(pool2, token).await?
            {
                (pool1, pool2)
            } else {
                (pool2, pool1)
            };
            Ok(ArbitrageOpportunity {
                token,
                profit_estimate: best_profit,
                path: vec![ArbitrageStep {
                    from_token: token,
                    to_token: token,
                    exchange: format!("Pool_{:?}", buy_pool.pair_address),
                    expected_rate: 1.0 + best_profit / 100.0,
                }],
                confidence: 0.7,
                expiration_block: self
                    .evm
                    .get_block_number()
                    .await
                    .map_err(|e| EvmError::Error(format!("{:?}", e)))
                    .unwrap()
                    + 10,
            })
        } else {
            Ok(ArbitrageOpportunity {
                token,
                profit_estimate: 0.0,
                path: vec![],
                confidence: 0.0,
                expiration_block: self
                    .evm
                    .get_block_number()
                    .await
                    .map_err(|e| EvmError::Error(format!("{:?}", e)))
                    .unwrap()
                    + 5,
            })
        }
    }

    async fn cal_triangular_profit(
        &self,
        steps: &[(&crate::liquidity::LiquidityPoolInfo, Address, Address)],
    ) -> Result<f64, EvmError> {
        let mut amount = 1.0;
        for &(pool, from, to) in steps {
            let rate = self.cal_exchange_rate(pool, from, to);
            amount *= rate;
        }
        Ok((amount - 1.0) * 100.0)
    }

    fn cal_exchange_rate(
        &self,
        pool: &crate::liquidity::LiquidityPoolInfo,
        from: Address,
        to: Address,
    ) -> f64 {
        if pool.token0 == from && pool.token1 == to {
            pool.reserve1.as_u128() as f64 / pool.reserve0.as_u128() as f64
        } else if pool.token1 == from && pool.token0 == to {
            pool.reserve0.as_u128() as f64 / pool.reserve1.as_u128() as f64
        } else {
            0.0
        }
    }

    async fn get_token_price_from_pool(
        &self,
        pool: &crate::liquidity::LiquidityPoolInfo,
        token: Address,
    ) -> Result<f64, EvmError> {
        if pool.token0 == token {
            Ok(pool.reserve1.as_u128() as f64 / pool.reserve0.as_u128() as f64)
        } else if pool.token1 == token {
            Ok(pool.reserve0.as_u128() as f64 / pool.reserve1.as_u128() as f64)
        } else {
            Err(EvmError::ContractError(
                "Token not found in pool".to_string(),
            ))
        }
    }

    async fn analyze_liquidity_metrics(
        &self,
        pool_address: Address,
    ) -> Result<LiquidityMetrics, EvmError> {
        let price_service = crate::price::Price::new(self.evm.clone());
        let (reserve0, reserve1, _) = price_service.get_reserves(pool_address).await?;
        let total_liquidity = reserve0.as_u128() as f64 + reserve1.as_u128() as f64;
        let liquidity_depth = (reserve0.as_u128() as f64 * reserve1.as_u128() as f64).sqrt();
        let score = Self::cal_liquidity_score(total_liquidity, liquidity_depth);
        Ok(LiquidityMetrics {
            score,
            total_liquidity,
            liquidity_depth,
        })
    }

    async fn analyze_volume_metrics(
        &self,
        pool_address: Address,
    ) -> Result<VolumeMetrics, EvmError> {
        let volume_24h = self.cal_24h_volume(pool_address).await?;
        let avg_volume = self.get_average_pool_volume().await;
        let volume_trend = if avg_volume > 0.0 {
            (volume_24h - avg_volume) / avg_volume
        } else {
            0.0
        };
        let score = Self::cal_volume_score(volume_24h, volume_trend);
        Ok(VolumeMetrics {
            score,
            volume_24h,
            volume_trend,
        })
    }

    async fn analyze_concentration_metrics(
        &self,
        pool_address: Address,
    ) -> Result<ConcentrationMetrics, EvmError> {
        let price_service = crate::price::Price::new(self.evm.clone());
        let (reserve0, reserve1, _) = price_service.get_reserves(pool_address).await?;
        let reserve_ratio = if !reserve1.is_zero() {
            reserve0.as_u128() as f64 / reserve1.as_u128() as f64
        } else {
            1.0
        };
        let impermanent_loss_risk = (reserve_ratio - 1.0).abs();
        let score = Self::cal_concentration_score(reserve_ratio, impermanent_loss_risk);
        Ok(ConcentrationMetrics {
            score,
            reserve_ratio,
            impermanent_loss_risk,
        })
    }

    async fn analyze_stability_metrics(
        &self,
        pool_address: Address,
    ) -> Result<StabilityMetrics, EvmError> {
        let price_history = crate::price::TokenPriceHistory::with_default_config(self.evm.clone());
        let stats = price_history
            .get_price_statistics(pool_address, EvmType::ETHEREUM_MAINNET)
            .await?;
        let price_volatility = stats.volatility_usd;
        let max_drawdown = self.cal_max_drawdown(pool_address).await?;
        let score = Self::cal_stability_score(price_volatility, max_drawdown);
        Ok(StabilityMetrics {
            score,
            price_volatility,
            max_drawdown,
        })
    }

    fn cal_liquidity_score(total_liquidity: f64, depth: f64) -> f64 {
        let liquidity_score = (total_liquidity.min(1_000_000.0) / 1_000_000.0) * 50.0;
        let depth_score = (depth.min(500_000.0) / 500_000.0) * 50.0;
        liquidity_score + depth_score
    }

    fn cal_volume_score(volume: f64, trend: f64) -> f64 {
        let volume_score = (volume.min(100_000.0) / 100_000.0) * 70.0;
        let trend_score = if trend > 0.0 { 30.0 } else { 0.0 };
        volume_score + trend_score
    }

    fn cal_concentration_score(ratio: f64, il_risk: f64) -> f64 {
        let ratio_score = if ratio >= 0.5 && ratio <= 2.0 {
            50.0
        } else if ratio >= 0.2 && ratio <= 5.0 {
            30.0
        } else {
            10.0
        };

        let risk_score = (1.0 - il_risk.min(1.0)) * 50.0;
        ratio_score + risk_score
    }

    fn cal_stability_score(volatility: f64, drawdown: f64) -> f64 {
        let volatility_score = (1.0 - volatility.min(1.0)) * 60.0;
        let drawdown_score = (1.0 - drawdown.min(1.0)) * 40.0;
        volatility_score + drawdown_score
    }

    async fn get_average_pool_volume(&self) -> f64 {
        static CACHE: Mutex<Option<(u64, f64)>> = Mutex::const_new(None);
        let now = chrono::Utc::now().timestamp() as u64;
        let cache_ttl = 300;
        {
            let cache = CACHE.lock().await;
            if let Some((timestamp, volume)) = *cache {
                if now - timestamp < cache_ttl {
                    return volume;
                }
            }
        }
        let average_volume = self.cal_average_volume_from_data().await;
        {
            let mut cache = CACHE.lock().await;
            *cache = Some((now, average_volume));
        }
        average_volume
    }

    async fn cal_average_volume_from_data(&self) -> f64 {
        let historical_volume = self.estimate_from_historical_data().await;
        let weighted_avg = (historical_volume * 0.3);
        weighted_avg.max(10_000.0)
    }

    async fn estimate_from_historical_data(&self) -> f64 {
        let blocks_per_day = 7200u64;
        let avg_swaps_per_block = 15.0;
        avg_swaps_per_block * blocks_per_day as f64 * 1000.0
    }

    async fn cal_max_drawdown(&self, pool_address: Address) -> Result<f64, EvmError> {
        let price_history = crate::price::TokenPriceHistory::with_default_config(self.evm.clone());
        let stats = price_history
            .get_price_statistics(pool_address, EvmType::ETHEREUM_MAINNET)
            .await?;
        Ok((stats.all_time_high_usd - stats.all_time_low_usd) / stats.all_time_high_usd)
    }

    fn generate_health_recommendations(
        &self,
        liquidity: &LiquidityMetrics,
        volume: &VolumeMetrics,
        concentration: &ConcentrationMetrics,
        stability: &StabilityMetrics,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        if liquidity.score < 60.0 {
            recommendations.push("Consider adding more liquidity to improve depth".to_string());
        }
        if volume.score < 50.0 {
            recommendations.push("Low trading volume may indicate illiquidity".to_string());
        }
        if concentration.impermanent_loss_risk > 0.1 {
            recommendations.push("High impermanent loss risk due to price volatility".to_string());
        }
        recommendations
    }
}

#[derive(Debug, Clone)]
struct LiquidityMetrics {
    score: f64,
    total_liquidity: f64,
    liquidity_depth: f64,
}

#[derive(Debug, Clone)]
struct VolumeMetrics {
    score: f64,
    volume_24h: f64,
    volume_trend: f64,
}

#[derive(Debug, Clone)]
struct ConcentrationMetrics {
    score: f64,
    reserve_ratio: f64,
    impermanent_loss_risk: f64,
}

#[derive(Debug, Clone)]
struct StabilityMetrics {
    score: f64,
    price_volatility: f64,
    max_drawdown: f64,
}

#[derive(Debug, Clone)]
pub struct DiscoveredPair {
    pub token0: Address,
    pub token1: Address,
    pub pair_address: Address,
    pub liquidity: f64,
    pub creation_block: u64,
}

#[derive(Debug, Clone)]
pub struct FeeEfficiencyReport {
    pub pool_address: Address,
    pub fee_earnings_24h: f64,
    pub lp_apr: f64,
    pub efficiency_score: f64,
    pub comparison: String,
}

pub struct FarmAnalyzer {
    evm: Arc<Evm>,
}

impl FarmAnalyzer {
    pub fn new(evm: Arc<Evm>) -> Self {
        Self { evm: evm }
    }

    pub async fn analyze_farm_performance(
        &self,
        farm_address: Address,
    ) -> Result<FarmPerformance, EvmError> {
        Ok(FarmPerformance {
            farm_address,
            total_rewards_distributed: U256::from(1000000u64),
            total_volume_24h: 500000.0,
            unique_stakers: 1500,
            average_stake_size: 2500.0,
            health_score: 85.5,
        })
    }

    pub async fn cal_optimal_strategy(
        &self,
        user_address: Address,
        available_lp_tokens: HashMap<Address, U256>,
        factory_address: Address,
    ) -> Result<Vec<FarmStrategy>, EvmError> {
        let all_farms = FarmService::new(self.evm.clone())
            .get_all_farms(factory_address, EvmType::ETHEREUM_MAINNET)
            .await?;
        let mut strategies = Vec::new();
        for (lp_token, amount) in available_lp_tokens {
            let relevant_farms: Vec<&FarmPool> = all_farms
                .iter()
                .filter(|farm| farm.staking_token == lp_token)
                .collect();
            for farm in relevant_farms {
                if amount > U256::zero() {
                    let expected_rewards = self
                        .cal_expected_rewards(farm.farm_address, user_address, amount)
                        .await?;

                    strategies.push(FarmStrategy {
                        farm_address: farm.farm_address,
                        lp_token,
                        stake_amount: amount,
                        expected_apr: farm.apr,
                        expected_daily_rewards: expected_rewards,
                        risk_score: self.cal_risk_score(farm).await?,
                    });
                }
            }
        }
        strategies.sort_by(|a, b| {
            b.expected_daily_rewards
                .partial_cmp(&a.expected_daily_rewards)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(strategies)
    }

    async fn cal_expected_rewards(
        &self,
        farm_address: Address,
        user_address: Address,
        stake_amount: U256,
    ) -> Result<f64, EvmError> {
        let farm_service = FarmService::new(self.evm.clone());
        let farm_detail = farm_service.get_farm_detail(farm_address).await?;
        if farm_detail.total_staked.is_zero() {
            return Ok(0.0);
        }
        let user_share = stake_amount.as_u128() as f64 / farm_detail.total_staked.as_u128() as f64;
        let daily_rewards = (farm_detail.reward_rate.as_u128() as f64 * 24.0 * 3600.0) / 1e18;
        Ok(daily_rewards * user_share)
    }

    async fn cal_risk_score(&self, farm: &FarmPool) -> Result<f64, EvmError> {
        let mut score = 100.0;
        if farm.tvl < 10000.0 {
            score -= 30.0;
        } else if farm.tvl < 50000.0 {
            score -= 15.0;
        }
        let time_remaining = farm.period_finish as i64 - chrono::Utc::now().timestamp();
        if time_remaining < 24 * 3600 {
            score -= 40.0;
        } else if time_remaining < 7 * 24 * 3600 {
            score -= 20.0;
        }
        if farm.apr > 1000.0 {
            score -= 25.0;
        } else if farm.apr > 500.0 {
            score -= 15.0;
        }
        Ok(f64::max(score, 0.0))
    }

    pub async fn detect_farm_risks(
        &self,
        farm_address: Address,
    ) -> Result<Vec<FarmRisk>, EvmError> {
        let farm_detail = FarmService::new(self.evm.clone())
            .get_farm_detail(farm_address)
            .await?;
        let mut risks = Vec::new();
        if farm_detail.tvl < 10000.0 {
            risks.push(FarmRisk {
                risk_type: "LOW_TVL".to_string(),
                severity: "HIGH".to_string(),
                description: "Farm TVL is very low, may be risky".to_string(),
                suggestion: "Consider farms with higher TVL".to_string(),
            });
        }
        let time_remaining = farm_detail.period_finish as i64 - chrono::Utc::now().timestamp();
        if time_remaining < 24 * 3600 {
            risks.push(FarmRisk {
                risk_type: "ENDING_SOON".to_string(),
                severity: "HIGH".to_string(),
                description: "Farm ends in less than 24 hours".to_string(),
                suggestion: "Consider unstaking before farm ends".to_string(),
            });
        }
        if farm_detail.apr > 1000.0 {
            risks.push(FarmRisk {
                risk_type: "UNSUSTAINABLE_APR".to_string(),
                severity: "MEDIUM".to_string(),
                description: "Extremely high APR may not be sustainable".to_string(),
                suggestion: "Monitor farm performance closely".to_string(),
            });
        }
        Ok(risks)
    }
}

#[derive(Debug, Clone)]
pub struct FarmStrategy {
    pub farm_address: Address,
    pub lp_token: Address,
    pub stake_amount: U256,
    pub expected_apr: f64,
    pub expected_daily_rewards: f64,
    pub risk_score: f64,
}

#[derive(Debug, Clone)]
pub struct FarmRisk {
    pub risk_type: String,
    pub severity: String, // LOW, MEDIUM, HIGH, CRITICAL
    pub description: String,
    pub suggestion: String,
}
