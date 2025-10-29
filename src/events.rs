use crate::EvmError;
use crate::price::Price;
use crate::tool::event_parsers::{
    parse_v4_liquidity_modified_log, parse_v4_pool_initialized_log, parse_v4_swap_log,
};
use crate::tool::{parse_burn_log, parse_mint_log, parse_pair_created_log, parse_swap_log};
use crate::types::{
    BurnEvent, FarmCreatedEvent, MintEvent, PairCreatedEvent, RewardClaimEvent, StakeEvent,
    SwapEvent, UnstakeEvent, V4LiquidityModifiedEvent,
};
use crate::types::{V4PoolInitializedEvent, V4SwapEvent};
use ethers::providers::Middleware;
use ethers::types::H256;
use ethers::types::{Address, Filter, ValueOrArray};
use evm_sdk::Evm;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use tokio::time::{Duration, MissedTickBehavior, interval};

/// Configuration for event listeners
#[derive(Debug, Clone)]
pub struct EventListenerConfig {
    pub poll_interval_secs: u64,
    pub max_blocks_per_poll: u64,
    pub confirmation_blocks: u64,
}

impl Default for EventListenerConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 5,
            max_blocks_per_poll: 1000,
            confirmation_blocks: 1,
        }
    }
}

struct EventListenerState {
    last_block_number: AtomicU64,
    is_running: AtomicBool,
}

// Event listener for Uniswap V2 and V3 events
pub struct UniswapEventListener {
    evm: Arc<Evm>,
    config: EventListenerConfig,
    state: Arc<EventListenerState>,
}

impl UniswapEventListener {
    /// Creates a new UniswapEventListener with default configuration
    pub fn new(evm: Arc<Evm>) -> Self {
        Self {
            evm,
            config: EventListenerConfig::default(),
            state: Arc::new(EventListenerState {
                last_block_number: AtomicU64::new(0),
                is_running: AtomicBool::new(false),
            }),
        }
    }

    /// Creates a new UniswapEventListener with custom configuration
    pub fn with_config(evm: Arc<Evm>, config: EventListenerConfig) -> Self {
        Self {
            evm,
            config,
            state: Arc::new(EventListenerState {
                last_block_number: AtomicU64::new(0),
                is_running: AtomicBool::new(false),
            }),
        }
    }

    /// Starts listening for Swap events
    ///
    /// # Example
    /// ```
    /// use ethers::types::Address;
    /// use std::str::FromStr;
    ///
    /// let listener = UniswapEventListener::new(client);
    /// let pair_addresses = vec![
    ///     Address::from_str("0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852").unwrap()
    /// ];
    ///
    /// listener.start_swap_listener(pair_addresses, |swap_event| {
    ///     println!("Swap detected: {:?}", swap_event);
    /// }).await;
    /// ```
    pub async fn start_swap_listener(
        &self,
        pair_addresses: Vec<Address>,
        on_swap: impl Fn(SwapEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(pair_addresses, "Swap".to_string(), move |log| {
            if let Ok(swap_event) = parse_swap_log(&log) {
                on_swap(swap_event);
            }
        })
        .await
    }

    /// Starts listening for PairCreated events
    pub async fn start_pair_created_listener(
        &self,
        factory_addresses: Vec<Address>,
        on_pair_created: impl Fn(PairCreatedEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(factory_addresses, "PairCreated".to_string(), move |log| {
            if let Ok(pair_event) = parse_pair_created_log(&log) {
                on_pair_created(pair_event);
            }
        })
        .await
    }

    /// Starts listening for Mint events
    pub async fn start_mint_listener(
        &self,
        pair_addresses: Vec<Address>,
        on_mint: impl Fn(MintEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(pair_addresses, "Mint".to_string(), move |log| {
            if let Ok(mint_event) = parse_mint_log(&log) {
                on_mint(mint_event);
            }
        })
        .await
    }

    /// Starts listening for Burn events
    pub async fn start_burn_listener(
        &self,
        pair_addresses: Vec<Address>,
        on_burn: impl Fn(BurnEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(pair_addresses, "Burn".to_string(), move |log| {
            if let Ok(burn_event) = parse_burn_log(&log) {
                on_burn(burn_event);
            }
        })
        .await
    }

    async fn start_listener(
        &self,
        addresses: Vec<Address>,
        event_name: String,
        callback: impl Fn(ethers::types::Log) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        if self.state.is_running.load(Ordering::SeqCst) {
            return Err(EvmError::ContractError(
                "Listener is already running".to_string(),
            ));
        }
        self.state.is_running.store(true, Ordering::SeqCst);
        if self.state.last_block_number.load(Ordering::SeqCst) == 0 {
            let current_block = self
                .evm
                .client
                .provider
                .get_block_number()
                .await
                .map_err(|e| {
                    EvmError::ContractError(format!("Failed to get current block: {}", e))
                })?;
            self.state
                .last_block_number
                .store(current_block.as_u64(), Ordering::SeqCst);
        }
        let client = self.evm.clone();
        let config = self.config.clone();
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.poll_interval_secs));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            while state.is_running.load(Ordering::SeqCst) {
                interval.tick().await;
                if let Err(e) = Self::poll_events(
                    &client,
                    state.clone(),
                    &addresses,
                    &event_name,
                    &callback,
                    &config,
                )
                .await
                {}
            }
        });
        Ok(())
    }

    async fn poll_events(
        evm: &Evm,
        state: Arc<EventListenerState>,
        addresses: &[Address],
        event_name: &str,
        callback: &impl Fn(ethers::types::Log),
        config: &EventListenerConfig,
    ) -> Result<(), EvmError> {
        let current_block =
            evm.client.provider.get_block_number().await.map_err(|e| {
                EvmError::ContractError(format!("Failed to get current block: {}", e))
            })?;
        let current_block_num = current_block.as_u64();
        let from_block = state.last_block_number.load(Ordering::SeqCst) + 1;
        let to_block = if current_block_num - from_block > config.max_blocks_per_poll {
            from_block + config.max_blocks_per_poll
        } else {
            current_block_num - config.confirmation_blocks
        };
        if from_block > to_block {
            return Ok(());
        }
        let filter = Filter::new()
            .address(ValueOrArray::Array(addresses.to_vec()))
            .from_block(from_block)
            .to_block(to_block)
            .event(event_name);
        let logs = evm
            .get_logs(filter)
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get logs: {}", e)))?;
        for log in logs {
            callback(log);
        }
        state.last_block_number.store(to_block, Ordering::SeqCst);
        Ok(())
    }

    /// Stops the event listener
    pub fn stop(&self) {
        self.state.is_running.store(false, Ordering::SeqCst);
    }

    /// Starts listening for Uniswap V4 Swap events
    pub async fn start_v4_swap_listener(
        &self,
        pool_manager_addresses: Vec<Address>,
        on_swap: impl Fn(V4SwapEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(
            pool_manager_addresses,
            "SwapExecuted".to_string(),
            move |log| {
                if let Ok(swap_event) = parse_v4_swap_log(&log) {
                    on_swap(swap_event);
                }
            },
        )
        .await
    }

    /// Starts listening for Uniswap V4 LiquidityModified events
    pub async fn start_v4_liquidity_modified_listener(
        &self,
        pool_manager_addresses: Vec<Address>,
        on_liquidity_modified: impl Fn(V4LiquidityModifiedEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(
            pool_manager_addresses,
            "LiquidityModified".to_string(),
            move |log| {
                if let Ok(event) = parse_v4_liquidity_modified_log(&log) {
                    on_liquidity_modified(event);
                }
            },
        )
        .await
    }

    /// Starts listening for Uniswap V4 PoolInitialized events
    pub async fn start_v4_pool_initialized_listener(
        &self,
        pool_manager_addresses: Vec<Address>,
        on_pool_initialized: impl Fn(V4PoolInitializedEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(
            pool_manager_addresses,
            "PoolInitialized".to_string(),
            move |log| {
                if let Ok(event) = parse_v4_pool_initialized_log(&log) {
                    on_pool_initialized(event);
                }
            },
        )
        .await
    }
}

/// Event listener for monitoring token price changes
pub struct TokenPriceEventListener {
    evm: Arc<Evm>,
    config: EventListenerConfig,
    state: Arc<EventListenerState>,
    price_threshold: f64,
}

impl TokenPriceEventListener {
    /// Creates a new TokenPriceEventListener with default configuration
    pub fn new(evm: Arc<Evm>, price_threshold: f64) -> Self {
        Self {
            evm,
            config: EventListenerConfig::default(),
            state: Arc::new(EventListenerState {
                last_block_number: AtomicU64::new(0),
                is_running: AtomicBool::new(false),
            }),
            price_threshold,
        }
    }

    /// Creates a new TokenPriceEventListener with custom configuration
    pub fn with_config(evm: Arc<Evm>, config: EventListenerConfig, price_threshold: f64) -> Self {
        Self {
            evm,
            config,
            state: Arc::new(EventListenerState {
                last_block_number: AtomicU64::new(0),
                is_running: AtomicBool::new(false),
            }),
            price_threshold,
        }
    }

    /// Starts monitoring token price changes
    ///
    /// # Example
    /// ```
    /// let price_listener = TokenPriceEventListener::new(client, 5.0);
    /// let token_addresses = vec![token0, token1];
    /// let pair_addresses = vec![pair_address];
    ///
    /// price_listener.start_price_change_listener(
    ///     token_addresses,
    ///     pair_addresses,
    ///     |price_event| {
    ///         println!("Price change detected: {}%", price_event.change_percent);
    ///     }
    /// ).await;
    /// ```
    pub async fn start_price_change_listener(
        &self,
        token_addresses: Vec<Address>,
        pair_addresses: Vec<Address>,
        on_price_change: impl Fn(TokenPriceChangeEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        if self.state.is_running.load(Ordering::SeqCst) {
            return Err(EvmError::ContractError(
                "Price change listener is already running".to_string(),
            ));
        }
        self.state.is_running.store(true, Ordering::SeqCst);
        if self.state.last_block_number.load(Ordering::SeqCst) == 0 {
            let current_block = self.evm.get_block_number().await.map_err(|e| {
                EvmError::ContractError(format!("Failed to get current block: {}", e))
            })?;
            self.state
                .last_block_number
                .store(current_block, Ordering::SeqCst);
        }
        let client = self.evm.clone();
        let config = self.config.clone();
        let state = self.state.clone();
        let price_threshold = self.price_threshold;
        let price_history = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.poll_interval_secs));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            while state.is_running.load(Ordering::SeqCst) {
                interval.tick().await;
                if let Err(e) = Self::monitor_price_changes(
                    &client,
                    state.clone(),
                    &token_addresses,
                    &pair_addresses,
                    &on_price_change,
                    &config,
                    price_threshold,
                    price_history.clone(),
                )
                .await
                {
                    eprintln!("Error monitoring price changes: {}", e);
                }
            }
        });
        Ok(())
    }

    async fn monitor_price_changes(
        evm: &Evm,
        state: Arc<EventListenerState>,
        token_addresses: &[Address],
        pair_addresses: &[Address],
        callback: &impl Fn(TokenPriceChangeEvent),
        config: &EventListenerConfig,
        price_threshold: f64,
        price_history: Arc<tokio::sync::Mutex<std::collections::HashMap<Address, f64>>>,
    ) -> Result<(), EvmError> {
        let current_block =
            evm.client.provider.get_block_number().await.map_err(|e| {
                EvmError::ContractError(format!("Failed to get current block: {}", e))
            })?;
        let current_block_num = current_block.as_u64();
        let from_block = state.last_block_number.load(Ordering::SeqCst) + 1;
        let to_block = current_block_num - config.confirmation_blocks;
        if from_block > to_block {
            return Ok(());
        }
        let filter = Filter::new()
            .address(ValueOrArray::Array(pair_addresses.to_vec()))
            .from_block(from_block)
            .to_block(to_block)
            .event("Swap");
        let logs = evm
            .get_logs(filter)
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get swap logs: {}", e)))?;

        for log in logs {
            if let Ok(swap_event) = parse_swap_log(&log) {
                if let Ok((token0, token1)) =
                    Self::get_pair_tokens(evm, swap_event.pair_address).await
                {
                    if let Ok(current_price) =
                        Self::calculate_token_price(&swap_event, token0, token1)
                    {
                        let mut history = price_history.lock().await;
                        let previous_price = history.get(&token0).copied();

                        if let Some(prev_price) = previous_price {
                            let price_change_percent =
                                ((current_price - prev_price) / prev_price) * 100.0;

                            if price_change_percent.abs() >= price_threshold {
                                let price_event = TokenPriceChangeEvent {
                                    token_address: token0,
                                    pair_address: swap_event.pair_address,
                                    previous_price: prev_price,
                                    current_price,
                                    change_percent: price_change_percent,
                                    block_number: swap_event.block_number,
                                    transaction_hash: swap_event.transaction_hash,
                                    direction: if price_change_percent > 0.0 {
                                        PriceChangeDirection::Up
                                    } else {
                                        PriceChangeDirection::Down
                                    },
                                };

                                callback(price_event);
                            }
                        }

                        history.insert(token0, current_price);
                    }
                }
            }
        }

        state.last_block_number.store(to_block, Ordering::SeqCst);

        Ok(())
    }

    /// Retrieves the token addresses for a given Uniswap V2 pair
    ///
    /// # Params
    ///
    /// client - Reference to the EVM client for blockchain interactions
    /// pair_address - Address of the Uniswap V2 pair contract
    ///
    /// # Returns
    ///
    /// Returns a tuple containing the addresses of both tokens in the pair:
    /// token0: First token in the pair (usually the token with lower address value)
    /// token1: Second token in the pair (usually the token with higher address value)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ethers::types::Address;
    /// use std::str::FromStr;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = EvmClient::new(EvmType::Ethereum).await?;
    /// let pair_address: Address = "0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852".parse()?; // WETH/USDT pair
    ///
    /// let (token0, token1) = get_pair_tokens(&client, pair_address).await?;
    /// println!("Token 0: {:?}, Token 1: {:?}", token0, token1);
    /// # Ok(())
    /// # }
    /// ```
    ///
    async fn get_pair_tokens(
        evm: &Evm,
        pair_address: Address,
    ) -> Result<(Address, Address), EvmError> {
        let price_service = Price::new(Arc::new(evm.clone()));
        let token0 = price_service.get_token0(pair_address).await?;
        let token1 = price_service.get_token1(pair_address).await?;
        Ok((token0, token1))
    }

    fn calculate_token_price(
        swap_event: &SwapEvent,
        token0: Address,
        token1: Address,
    ) -> Result<f64, EvmError> {
        if !swap_event.amount0_in.is_zero() && !swap_event.amount1_out.is_zero() {
            let input = swap_event.amount0_in.as_u128() as f64;
            let output = swap_event.amount1_out.as_u128() as f64;
            if input > 0.0 {
                Ok(output / input)
            } else {
                Err(EvmError::ContractError("Invalid swap amounts".to_string()))
            }
        } else if !swap_event.amount1_in.is_zero() && !swap_event.amount0_out.is_zero() {
            let input = swap_event.amount1_in.as_u128() as f64;
            let output = swap_event.amount0_out.as_u128() as f64;
            if input > 0.0 {
                Ok(input / output)
            } else {
                Err(EvmError::ContractError("Invalid swap amounts".to_string()))
            }
        } else {
            Err(EvmError::ContractError(
                "Cannot determine price from swap event".to_string(),
            ))
        }
    }

    /// Stops the price change listener
    pub fn stop(&self) {
        self.state.is_running.store(false, Ordering::SeqCst);
    }
}

/// uniswap event manager
pub struct UniswapEventManager {
    evm: Arc<Evm>,
}
impl UniswapEventManager {
    /// Creates a new UniswapEventManager
    pub fn new(evm: Arc<Evm>) -> Self {
        Self { evm: evm.clone() }
    }

    /// Retrieves Swap events within a block range
    ///
    /// # Example
    /// ```
    /// let event_manager = UniswapEventManager::new(client);
    /// let swap_events = event_manager.get_swap_events_in_range(
    ///     &pair_addresses,
    ///     18000000,
    ///     18000100
    /// ).await?;
    /// ```
    async fn get_swap_events_in_range(
        &self,
        pair_addresses: &[Address],
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<SwapEvent>, EvmError> {
        let logs = self
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

    /// Retrieves historical events
    pub async fn get_historical_events(
        &self,
        addresses: Vec<Address>,
        event_name: &str,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<ethers::types::Log>, EvmError> {
        let filter = Filter::new()
            .address(ValueOrArray::Array(addresses))
            .from_block(from_block)
            .to_block(to_block)
            .event(event_name);
        self.evm
            .get_logs(filter)
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get historical events: {}", e)))
    }
}

/// Represents a token price change event
#[derive(Debug, Clone)]
pub struct TokenPriceChangeEvent {
    pub token_address: Address,
    pub pair_address: Address,
    pub previous_price: f64,
    pub current_price: f64,
    pub change_percent: f64,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub direction: PriceChangeDirection,
}

/// Direction of price change
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PriceChangeDirection {
    Up,
    Down,
}

/// Event listener for farm-related events
pub struct FarmEventListener {
    evm: Arc<Evm>,
    is_running: Arc<AtomicBool>,
    last_block: Arc<AtomicU64>,
}

impl FarmEventListener {
    /// Creates a new FarmEventListener
    pub fn new(evm: Arc<Evm>) -> Self {
        Self {
            evm,
            is_running: Arc::new(AtomicBool::new(false)),
            last_block: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Starts listening for stake events
    ///
    /// # Example
    /// ```
    /// let farm_listener = FarmEventListener::new(client);
    /// farm_listener.start_stake_listener(
    ///     farm_addresses,
    ///     |stake_event| {
    ///         println!("User {} staked {}", stake_event.user, stake_event.amount);
    ///     }
    /// ).await;
    /// ```
    pub async fn start_stake_listener(
        &self,
        farm_addresses: Vec<Address>,
        on_stake: impl Fn(StakeEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(farm_addresses, "Staked".to_string(), move |log| {
            if let Ok(stake_event) = Self::parse_stake_log(&log) {
                on_stake(stake_event);
            }
        })
        .await
    }

    /// Starts listening for unstake events
    pub async fn start_unstake_listener(
        &self,
        farm_addresses: Vec<Address>,
        on_unstake: impl Fn(UnstakeEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(farm_addresses, "Withdrawn".to_string(), move |log| {
            if let Ok(unstake_event) = Self::parse_unstake_log(&log) {
                on_unstake(unstake_event);
            }
        })
        .await
    }

    /// Starts listening for reward claim events
    pub async fn start_reward_claim_listener(
        &self,
        farm_addresses: Vec<Address>,
        on_claim: impl Fn(RewardClaimEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(farm_addresses, "RewardPaid".to_string(), move |log| {
            if let Ok(claim_event) = Self::parse_reward_claim_log(&log) {
                on_claim(claim_event);
            }
        })
        .await
    }

    /// Starts listening for farm creation events
    pub async fn start_farm_created_listener(
        &self,
        factory_addresses: Vec<Address>,
        on_farm_created: impl Fn(FarmCreatedEvent) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        self.start_listener(factory_addresses, "FarmCreated".to_string(), move |log| {
            if let Ok(farm_event) = Self::parse_farm_created_log(&log) {
                on_farm_created(farm_event);
            }
        })
        .await
    }

    async fn start_listener(
        &self,
        addresses: Vec<Address>,
        event_name: String,
        callback: impl Fn(ethers::types::Log) + Send + Sync + 'static,
    ) -> Result<(), EvmError> {
        if self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(EvmError::ContractError(
                "Listener is already running".to_string(),
            ));
        }
        self.is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);
        if self.last_block.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            let current_block = self
                .evm
                .client
                .provider
                .get_block_number()
                .await
                .map_err(|e| {
                    EvmError::ContractError(format!("Failed to get current block: {}", e))
                })?;
            self.last_block
                .store(current_block.as_u64(), std::sync::atomic::Ordering::SeqCst);
        }
        let evm = self.evm.clone();
        let is_running = self.is_running.clone();
        let last_block = self.last_block.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                interval.tick().await;
                if let Err(e) =
                    Self::poll_events(&evm, &last_block, &addresses, &event_name, &callback).await
                {
                    eprintln!("Error polling farm events: {}", e);
                }
            }
        });
        Ok(())
    }

    async fn poll_events(
        evm: &Evm,
        last_block: &AtomicU64,
        addresses: &[Address],
        event_name: &str,
        callback: &impl Fn(ethers::types::Log),
    ) -> Result<(), EvmError> {
        let current_block =
            evm.client.provider.get_block_number().await.map_err(|e| {
                EvmError::ContractError(format!("Failed to get current block: {}", e))
            })?;
        let from_block = last_block.load(std::sync::atomic::Ordering::SeqCst) + 1;
        let to_block = current_block.as_u64();
        if from_block > to_block {
            return Ok(());
        }
        let filter = Filter::new()
            .address(ValueOrArray::Array(addresses.to_vec()))
            .from_block(from_block)
            .to_block(to_block)
            .event(event_name);
        let logs = evm.get_logs(filter).await.map_err(|e| {
            EvmError::ContractError(format!("Failed to get farm event logs: {}", e))
        })?;
        for log in logs {
            callback(log);
        }
        last_block.store(to_block, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    fn parse_stake_log(log: &ethers::types::Log) -> Result<StakeEvent, EvmError> {
        if log.topics.len() < 2 {
            return Err(EvmError::ContractError(
                "Invalid Staked event log".to_string(),
            ));
        }
        let user = Address::from_slice(&log.topics[1].as_bytes()[12..]);
        if log.data.len() < 32 {
            return Err(EvmError::ContractError(
                "Invalid Staked event data length".to_string(),
            ));
        }
        let amount = ethers::types::U256::from_big_endian(&log.data[0..32]);
        Ok(StakeEvent {
            farm_address: log.address,
            user,
            amount,
            block_number: log.block_number.unwrap().as_u64(),
            transaction_hash: log.transaction_hash.unwrap(),
        })
    }

    fn parse_unstake_log(log: &ethers::types::Log) -> Result<UnstakeEvent, EvmError> {
        if log.topics.len() < 2 {
            return Err(EvmError::ContractError(
                "Invalid Withdrawn event log".to_string(),
            ));
        }
        let user = Address::from_slice(&log.topics[1].as_bytes()[12..]);
        if log.data.len() < 32 {
            return Err(EvmError::ContractError(
                "Invalid Withdrawn event data length".to_string(),
            ));
        }
        let amount = ethers::types::U256::from_big_endian(&log.data[0..32]);
        Ok(UnstakeEvent {
            farm_address: log.address,
            user,
            amount,
            block_number: log.block_number.unwrap().as_u64(),
            transaction_hash: log.transaction_hash.unwrap(),
        })
    }

    fn parse_reward_claim_log(log: &ethers::types::Log) -> Result<RewardClaimEvent, EvmError> {
        if log.topics.len() < 2 {
            return Err(EvmError::ContractError(
                "Invalid RewardPaid event log".to_string(),
            ));
        }
        let user = Address::from_slice(&log.topics[1].as_bytes()[12..]);
        if log.data.len() < 32 {
            return Err(EvmError::ContractError(
                "Invalid RewardPaid event data length".to_string(),
            ));
        }
        let reward_amount = ethers::types::U256::from_big_endian(&log.data[0..32]);
        Ok(RewardClaimEvent {
            farm_address: log.address,
            user,
            reward_amount,
            block_number: log.block_number.unwrap().as_u64(),
            transaction_hash: log.transaction_hash.unwrap(),
        })
    }

    fn parse_farm_created_log(log: &ethers::types::Log) -> Result<FarmCreatedEvent, EvmError> {
        if log.topics.len() < 4 {
            return Err(EvmError::ContractError(
                "Invalid FarmCreated event log".to_string(),
            ));
        }
        let staking_token = Address::from_slice(&log.topics[1].as_bytes()[12..]);
        let reward_token = Address::from_slice(&log.topics[2].as_bytes()[12..]);
        let farm_address = Address::from_slice(&log.topics[3].as_bytes()[12..]);
        if log.data.len() < 32 {
            return Err(EvmError::ContractError(
                "Invalid FarmCreated event data length".to_string(),
            ));
        }
        let rewards_duration = ethers::types::U256::from_big_endian(&log.data[0..32]);
        Ok(FarmCreatedEvent {
            farm_address,
            staking_token,
            reward_token,
            rewards_duration: rewards_duration.as_u64(),
            block_number: log.block_number.unwrap().as_u64(),
            transaction_hash: log.transaction_hash.unwrap(),
        })
    }

    /// Stops the farm event listener
    pub fn stop(&self) {
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
