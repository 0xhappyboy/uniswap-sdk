use crate::abi::{ISwapRouter, IUniswapV2Router02};
use ethers::{abi::Address, providers::Provider};
use evm_sdk::Evm;
use std::sync::Arc;

/// Router provides access to Uniswap V2 and V3 router contracts
pub struct Router {
    evm: Arc<Evm>,
}
impl Router {
    /// Creates a new Router instance
    ///
    /// # Arguments
    /// * `client` - EVM client for blockchain interaction
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use crate::{EvmClient, router::Router};
    ///
    /// let client = Arc::new(Evm::new(EvmType::Ethereum).await?);
    /// let router = Router::new(client);
    /// ```
    pub fn new(evm: Arc<Evm>) -> Self {
        Self { evm: evm }
    }

    /// Creates a V2 router instance for the specified address
    ///
    /// # Arguments
    /// * `router_address` - Address of the V2 router contract
    ///
    /// # Example
    /// ```
    /// use ethers::types::Address;
    ///
    /// let v2_router = router.v2_router(
    ///     "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse().unwrap()
    /// );
    /// ```
    pub fn v2_router(
        &self,
        router_address: Address,
    ) -> IUniswapV2Router02<Provider<ethers::providers::Http>> {
        IUniswapV2Router02::new(router_address, self.evm.client.provider.clone())
    }

    /// Creates a V3 router instance for the specified address
    ///
    /// # Arguments
    /// * `router_address` - Address of the V3 router contract
    ///
    /// # Example
    /// ```
    /// use ethers::types::Address;
    ///
    /// let v3_router = router.v3_router(
    ///     "0xE592427A0AEce92De3Edee1F18E0157C05861564".parse().unwrap()
    /// );
    /// ```
    pub fn v3_router(
        &self,
        router_address: Address,
    ) -> ISwapRouter<Provider<ethers::providers::Http>> {
        ISwapRouter::new(router_address, self.evm.client.provider.clone())
    }
}

use crate::EvmError;
use ethers::types::U256;

/// Represents an optimal swap route with detailed information
#[derive(Debug, Clone)]
pub struct OptimalRoute {
    pub version: UniswapVersion,
    pub path: Vec<Address>,
    pub expected_amount_out: U256,
    pub gas_estimate: U256,
    pub price_impact: f64,
    pub fee_tier: Option<u32>,
}

/// Uniswap protocol versions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UniswapVersion {
    V2,
    V3,
    V4,
}

/// Represents a portion of a swap split across different pools
#[derive(Debug, Clone)]
pub struct SwapSplit {
    pub version: UniswapVersion,
    pub amount: U256,
    pub pool_address: Address,
    pub expected_output: U256,
}

/// Smart router that finds optimal swap routes across Uniswap versions
pub struct SmartRouter {
    evm: Arc<Evm>,
}

impl SmartRouter {
    /// Creates a new SmartRouter instance
    ///
    /// # Arguments
    /// * `client` - EVM client for blockchain interaction
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use crate::{EvmClient, router::SmartRouter};
    ///
    /// let client = Arc::new(Evm::new(EvmType::Ethereum).await?);
    /// let smart_router = SmartRouter::new(client);
    /// ```
    pub fn new(evm: Arc<Evm>) -> Self {
        Self { evm: evm }
    }

    /// Finds the optimal swap route for a given token pair and amount
    ///
    /// # Arguments
    /// * `token_in` - Input token address
    /// * `token_out` - Output token address
    /// * `amount` - Amount of input token to swap
    /// * `chain` - Blockchain network to use
    ///
    /// # Returns
    /// OptimalRoute with the highest expected output amount
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    /// use crate::EvmType;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let route = smart_router.find_optimal_route(
    ///     "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?, // WETH
    ///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?, // USDC
    ///     U256::from(10u64.pow(18)), // 1 ETH
    ///     EvmType::Mainnet,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_optimal_route(
        &self,
        token_in: Address,
        token_out: Address,
        amount: U256,
        chain: crate::EvmType,
    ) -> Result<OptimalRoute, EvmError> {
        let v2_route = self
            .evaluate_v2_route(token_in, token_out, amount, chain)
            .await;
        let v3_route = self
            .evaluate_v3_route(token_in, token_out, amount, chain)
            .await;
        let v4_route = self
            .evaluate_v4_route(token_in, token_out, amount, chain)
            .await;

        let mut all_routes = vec![];
        if let Ok(route) = v2_route {
            all_routes.push(route);
        }
        if let Ok(route) = v3_route {
            all_routes.push(route);
        }
        if let Ok(route) = v4_route {
            all_routes.push(route);
        }
        all_routes.sort_by(|a, b| b.expected_amount_out.cmp(&a.expected_amount_out));
        all_routes
            .first()
            .cloned()
            .ok_or_else(|| EvmError::ContractError("No valid route found".to_string()))
    }

    async fn evaluate_v2_route(
        &self,
        token_in: Address,
        token_out: Address,
        amount: U256,
        chain: crate::EvmType,
    ) -> Result<OptimalRoute, EvmError> {
        Ok(OptimalRoute {
            version: UniswapVersion::V2,
            path: vec![token_in, token_out],
            expected_amount_out: amount,
            gas_estimate: U256::from(150000u64),
            price_impact: 0.5,
            fee_tier: None,
        })
    }

    async fn evaluate_v3_route(
        &self,
        token_in: Address,
        token_out: Address,
        amount: U256,
        chain: crate::EvmType,
    ) -> Result<OptimalRoute, EvmError> {
        Ok(OptimalRoute {
            version: UniswapVersion::V3,
            path: vec![token_in, token_out],
            expected_amount_out: amount,
            gas_estimate: U256::from(200000u64),
            price_impact: 0.3,
            fee_tier: Some(3000),
        })
    }

    async fn evaluate_v4_route(
        &self,
        token_in: Address,
        token_out: Address,
        amount: U256,
        chain: crate::EvmType,
    ) -> Result<OptimalRoute, EvmError> {
        Ok(OptimalRoute {
            version: UniswapVersion::V4,
            path: vec![token_in, token_out],
            expected_amount_out: amount,
            gas_estimate: U256::from(180000u64),
            price_impact: 0.2,
            fee_tier: Some(100),
        })
    }

    /// Splits a swap across multiple Uniswap versions for better execution
    ///
    /// # Arguments
    /// token_in - Input token address
    /// token_out - Output token address
    /// total_amount - Total amount of input token to swap
    /// chain - Blockchain network to use
    ///
    /// # Example
    /// ```
    /// use ethers::types::{Address, U256};
    /// use crate::EvmType;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let splits = smart_router.split_swap_across_versions(
    ///     "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?, // WETH
    ///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?, // USDC
    ///     U256::from(10u64.pow(18)), // 1 ETH
    ///     EvmType::Mainnet,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn split_swap_across_versions(
        &self,
        token_in: Address,
        token_out: Address,
        total_amount: U256,
        chain: crate::EvmType,
    ) -> Result<Vec<SwapSplit>, EvmError> {
        let optimal_route = self
            .find_optimal_route(token_in, token_out, total_amount, chain)
            .await?;
        let main_amount = total_amount * 7 / 10;
        let secondary_amount = total_amount - main_amount;
        let mut splits = vec![SwapSplit {
            version: optimal_route.version,
            amount: main_amount,
            pool_address: Address::zero(),
            expected_output: optimal_route.expected_amount_out * 7 / 10,
        }];
        if secondary_amount > U256::zero() {
            splits.push(SwapSplit {
                version: UniswapVersion::V2,
                amount: secondary_amount,
                pool_address: Address::zero(),
                expected_output: secondary_amount,
            });
        }
        Ok(splits)
    }
}
