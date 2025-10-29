use std::fmt;

use ethers::{
    abi::Address,
    types::{H256, U256},
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum EvmError {
    ConfigError(String),
    ConnectionError(String),
    RpcError(String),
    WalletError(String),
    TransactionError(String),
    ContractError(String),
    InvalidInput(String),
    IOError(String),
    AaveError(String),
    ListenerError(String),
    ProviderError(String),
    CalculationError(String),
    MempoolError(String),
    Error(String),
}

impl fmt::Display for EvmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            EvmError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            EvmError::RpcError(msg) => write!(f, "RPC error: {}", msg),
            EvmError::WalletError(msg) => write!(f, "Wallet error: {}", msg),
            EvmError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            EvmError::ContractError(msg) => write!(f, "Contract error: {}", msg),
            EvmError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            EvmError::IOError(msg) => write!(f, "IO Error: {}", msg),
            EvmError::AaveError(msg) => write!(f, "Aave Error: {}", msg),
            EvmError::ListenerError(msg) => write!(f, "Aave Error: {}", msg),
            EvmError::ProviderError(msg) => write!(f, "Aave Error: {}", msg),
            EvmError::CalculationError(msg) => write!(f, "Aave Error: {}", msg),
            EvmError::MempoolError(msg) => write!(f, "Aave Error: {}", msg),
            EvmError::Error(msg) => write!(f, "Aave Error: {}", msg),
        }
    }
}

impl std::error::Error for EvmError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvmType {
    Ethereum,
    Arb,
    Bsc,
    Base,
    HyperEVM,
    Plasma,
}

impl EvmType {
    pub fn name(&self) -> &'static str {
        match self {
            EvmType::Ethereum => "Ethereum",
            EvmType::Arb => "Arbitrum",
            EvmType::Bsc => "Binance Smart Chain",
            EvmType::Base => "Base",
            EvmType::HyperEVM => "HyperEVM",
            EvmType::Plasma => "Plasma",
        }
    }

    pub fn chain_id(&self) -> u64 {
        match self {
            EvmType::Ethereum => 1,
            EvmType::Arb => 42161,
            EvmType::Bsc => 56,
            EvmType::Base => 8453,
            EvmType::HyperEVM => 777,
            EvmType::Plasma => 94,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapEvent {
    pub pair_address: Address,
    pub sender: Address,
    pub to: Address,
    pub amount0_in: U256,
    pub amount1_in: U256,
    pub amount0_out: U256,
    pub amount1_out: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub log_index: u64,
}

#[derive(Debug, Clone)]
pub struct PairCreatedEvent {
    pub factory_address: Address,
    pub token0: Address,
    pub token1: Address,
    pub pair: Address,
    pub pair_count: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub log_index: u64,
}

#[derive(Debug, Clone)]
pub struct MintEvent {
    pub pair_address: Address,
    pub sender: Address,
    pub amount0: U256,
    pub amount1: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub log_index: u64,
}

#[derive(Debug, Clone)]
pub struct BurnEvent {
    pub pair_address: Address,
    pub sender: Address,
    pub to: Address,
    pub amount0: U256,
    pub amount1: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub log_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V4PoolInfo {
    pub pool_address: Address,
    pub currency0: Address,
    pub currency1: Address,
    pub fee: u32,
    pub tick_spacing: i32,
    pub hooks: Address,
    pub sqrt_price_x96: U256,
    pub current_tick: i32,
    pub liquidity: U256,
    pub fee_growth_global0_x128: U256,
    pub fee_growth_global1_x128: U256,
    pub protocol_fees_token0: U256,
    pub protocol_fees_token1: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V4PositionInfo {
    pub position_key: H256,
    pub owner: Address,
    pub liquidity: U256,
    pub fee_growth_inside0_last_x128: U256,
    pub fee_growth_inside1_last_x128: U256,
    pub tokens_owed0: U256,
    pub tokens_owed1: U256,
}

#[derive(Debug, Clone)]
pub struct V4SwapEvent {
    pub pool_address: Address,
    pub sender: Address,
    pub amount0: i128,
    pub amount1: i128,
    pub sqrt_price_x96: U256,
    pub liquidity: U256,
    pub tick: i32,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub log_index: u64,
}

#[derive(Debug, Clone)]
pub struct V4LiquidityModifiedEvent {
    pub pool_address: Address,
    pub owner: Address,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity_delta: i128,
    pub amount0: i128,
    pub amount1: i128,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub log_index: u64,
}

#[derive(Debug, Clone)]
pub struct V4PoolInitializedEvent {
    pub pool_address: Address,
    pub currency0: Address,
    pub currency1: Address,
    pub fee: u32,
    pub tick_spacing: i32,
    pub hooks: Address,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub log_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickInfo {
    pub liquidity_gross: U256,
    pub liquidity_net: i128,
    pub fee_growth_outside0_x128: U256,
    pub fee_growth_outside1_x128: U256,
    pub tick_cumulative_outside: i64,
    pub seconds_per_liquidity_outside_x128: U256,
    pub seconds_outside: u32,
    pub initialized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmPool {
    pub farm_address: Address,
    pub staking_token: Address,
    pub reward_token: Address,
    pub staking_token_symbol: String,
    pub reward_token_symbol: String,
    pub total_staked: U256,
    pub reward_rate: U256,
    pub rewards_duration: u64,
    pub period_finish: u64,
    pub last_update_time: u64,
    pub reward_per_token_stored: U256,
    pub apr: f64,
    pub tvl: f64,
    pub is_active: bool,
    pub created_block: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFarmPosition {
    pub user_address: Address,
    pub farm_address: Address,
    pub staked_amount: U256,
    pub pending_rewards: U256,
    pub reward_debt: U256,
    pub last_update_time: u64,
    pub total_earned: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmRewardInfo {
    pub reward_token: Address,
    pub reward_rate: U256,
    pub period_finish: u64,
    pub rewards_duration: u64,
    pub last_update_time: u64,
    pub reward_per_token_stored: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmAPY {
    pub farm_address: Address,
    pub apr_7d: f64,
    pub apr_30d: f64,
    pub apy_7d: f64,
    pub apy_30d: f64,
    pub daily_rewards: f64,
    pub weekly_rewards: f64,
}

#[derive(Debug, Clone)]
pub struct StakeEvent {
    pub farm_address: Address,
    pub user: Address,
    pub amount: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
}

#[derive(Debug, Clone)]
pub struct UnstakeEvent {
    pub farm_address: Address,
    pub user: Address,
    pub amount: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
}

#[derive(Debug, Clone)]
pub struct RewardClaimEvent {
    pub farm_address: Address,
    pub user: Address,
    pub reward_amount: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
}

#[derive(Debug, Clone)]
pub struct FarmCreatedEvent {
    pub farm_address: Address,
    pub staking_token: Address,
    pub reward_token: Address,
    pub rewards_duration: u64,
    pub block_number: u64,
    pub transaction_hash: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmStakingHistory {
    pub timestamp: u64,
    pub total_staked: U256,
    pub reward_rate: U256,
    pub apr: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmPerformance {
    pub farm_address: Address,
    pub total_rewards_distributed: U256,
    pub total_volume_24h: f64,
    pub unique_stakers: u64,
    pub average_stake_size: f64,
    pub health_score: f64,
}
