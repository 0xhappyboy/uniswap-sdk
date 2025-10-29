use crate::Arc;
use crate::Evm;
use crate::EvmError;
use crate::UniswapConfig;
use crate::abi::IUniswapV2Factory;
use ethers::providers::Provider;
use ethers::types::Address;
use evm_client::EvmType;

/// Factory for interacting with Uniswap V2 and V3 factories
/// Provides methods to discover trading pairs and liquidity pools
pub struct Factory {
    evm: Arc<Evm>,
}
impl Factory {
    /// Creates a new Factory instance
    ///
    /// # Params
    /// client` - EVM client for blockchain interactions
    ///
    /// # Example
    /// ```
    /// use crate::{Arc, Factory};
    ///
    /// let client = Evm::new(EvmType::ETHEREUM_MAINNET).await?;
    /// let factory = Factory::new(client);
    /// ```
    pub fn new(evm: Arc<Evm>) -> Self {
        Self { evm: evm }
    }

    /// Get Uniswap V2 Factory instance
    fn v2_factory(
        &self,
        factory_address: Address,
    ) -> IUniswapV2Factory<Provider<ethers::providers::Http>> {
        IUniswapV2Factory::new(factory_address, self.evm.client.provider.clone())
    }

    /// Finds common price pairs for a given token across stablecoins and ETH
    /// Returns tuple containing (usd_pair_v2, eth_pair_v2, v3_pool_usd, v3_pool_eth)
    ///
    /// # Params
    /// token_address - Address of the token to find pairs for
    /// chain - EVM chain type (Mainnet, Polygon, etc.)
    ///
    /// # Example
    /// ```rust
    /// use crate::{Arc, EvmClient, Factory, EvmType};
    /// use ethers::types::Address;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = EvmClient::new(EvmType::Ethereum).await?;
    /// let factory = Factory::new(client);
    /// let token_addr: Address = "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984".parse()?; // UNI token
    ///
    /// let pairs = factory.find_price_pairs(token_addr, EvmType::Mainnet).await?;
    /// println!("USD pair: {:?}, ETH pair: {:?}", pairs.0, pairs.1);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_price_pairs(
        &self,
        token_address: Address,
        chain: EvmType,
    ) -> Result<
        (
            Option<Address>,
            Option<Address>,
            Option<Address>,
            Option<Address>,
        ),
        EvmError,
    > {
        let factory_v2 = UniswapConfig::v2_factory_address(chain)?;
        // Common stablecoin addresses (you may need to adjust these per chain)
        let usdc_address =
            crate::tool::address::str_to_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
                .unwrap();
        let usdt_address =
            crate::tool::address::str_to_address("0xdAC17F958D2ee523a2206206994597C13D831ec7")
                .unwrap();
        let dai_address =
            crate::tool::address::str_to_address("0x6B175474E89094C44Da98b954EedeAC495271d0F")
                .unwrap();
        let weth_address =
            crate::tool::address::str_to_address("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
                .unwrap();
        let mut usd_pair_v2 = None;
        let mut eth_pair_v2 = None;
        // Try to find USD pairs (with stablecoins)
        for &stablecoin in &[usdc_address, usdt_address, dai_address] {
            if let Ok(pair) = self
                .get_pair_address(factory_v2, token_address, stablecoin)
                .await
            {
                if !pair.is_zero() {
                    usd_pair_v2 = Some(pair);
                    break;
                }
            }
        }
        // Try to find ETH pair
        if let Ok(pair) = self
            .get_pair_address(factory_v2, token_address, weth_address)
            .await
        {
            if !pair.is_zero() {
                eth_pair_v2 = Some(pair);
            }
        }
        // For V3, you would need to implement similar logic using V3 factory
        // This is a simplified version - in production you'd want proper V3 pool discovery
        let v3_pool_usd = None; // Implement V3 pool discovery as needed
        let v3_pool_eth = None; // Implement V3 pool discovery as needed
        Ok((usd_pair_v2, eth_pair_v2, v3_pool_usd, v3_pool_eth))
    }

    /// Get pair address for two tokens
    /// Gets the pair address for two tokens from Uniswap V2 factory
    ///
    /// # Params
    /// factory_address - Address of the Uniswap V2 factory contract
    /// token_a - Address of first token in the pair
    /// token_b - Address of second token in the pair
    ///
    /// # Example
    /// ```
    /// use crate::{Arc, EvmClient, Factory};
    /// use ethers::types::Address;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = EvmClient::new(EvmType::Ethereum).await?;
    /// let factory = Factory::new(client);
    /// let factory_addr: Address = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".parse()?;
    /// let weth: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?;
    /// let dai: Address = "0x6B175474E89094C44Da98b954EedeAC495271d0F".parse()?;
    ///
    /// let pair_addr = factory.get_pair_address(factory_addr, weth, dai).await?;
    /// println!("WETH/DAI pair: {}", pair_addr);
    /// Ok(())
    /// }
    /// ```
    pub async fn get_pair_address(
        &self,
        factory_address: Address,
        token_a: Address,
        token_b: Address,
    ) -> Result<Address, EvmError> {
        let factory = self.v2_factory(factory_address);
        factory
            .get_pair(token_a, token_b)
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get pair address: {}", e)))
    }
}
