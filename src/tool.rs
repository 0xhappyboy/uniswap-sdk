use crate::EvmError;
use crate::types::{BurnEvent, MintEvent, PairCreatedEvent, SwapEvent};
use ethers::types::{Address, U256};

pub fn parse_swap_log(log: &ethers::types::Log) -> Result<SwapEvent, EvmError> {
    if log.topics.len() < 3 {
        return Err(EvmError::ContractError(
            "Invalid Swap event log".to_string(),
        ));
    }
    let sender = Address::from_slice(&log.topics[1].as_bytes()[12..]);
    let to = Address::from_slice(&log.topics[2].as_bytes()[12..]);
    if log.data.len() < 128 {
        return Err(EvmError::ContractError(
            "Invalid Swap event data length".to_string(),
        ));
    }
    let amount0_in = U256::from_big_endian(&log.data[0..32]);
    let amount1_in = U256::from_big_endian(&log.data[32..64]);
    let amount0_out = U256::from_big_endian(&log.data[64..96]);
    let amount1_out = U256::from_big_endian(&log.data[96..128]);
    Ok(SwapEvent {
        pair_address: log.address,
        sender,
        to,
        amount0_in,
        amount1_in,
        amount0_out,
        amount1_out,
        block_number: log.block_number.unwrap().as_u64(),
        transaction_hash: log.transaction_hash.unwrap(),
        log_index: log.log_index.unwrap().as_u64(),
    })
}

pub fn parse_pair_created_log(log: &ethers::types::Log) -> Result<PairCreatedEvent, EvmError> {
    if log.topics.len() < 3 {
        return Err(EvmError::ContractError(
            "Invalid PairCreated event log".to_string(),
        ));
    }
    let token0 = Address::from_slice(&log.topics[1].as_bytes()[12..]);
    let token1 = Address::from_slice(&log.topics[2].as_bytes()[12..]);
    if log.data.len() < 64 {
        return Err(EvmError::ContractError(
            "Invalid PairCreated event data length".to_string(),
        ));
    }
    let pair = Address::from_slice(&log.data[12..32]);
    let pair_count = U256::from_big_endian(&log.data[32..64]);
    Ok(PairCreatedEvent {
        factory_address: log.address,
        token0,
        token1,
        pair,
        pair_count,
        block_number: log.block_number.unwrap().as_u64(),
        transaction_hash: log.transaction_hash.unwrap(),
        log_index: log.log_index.unwrap().as_u64(),
    })
}

pub fn parse_mint_log(log: &ethers::types::Log) -> Result<MintEvent, EvmError> {
    if log.topics.len() < 2 {
        return Err(EvmError::ContractError(
            "Invalid Mint event log".to_string(),
        ));
    }
    let sender = Address::from_slice(&log.topics[1].as_bytes()[12..]);
    if log.data.len() < 64 {
        return Err(EvmError::ContractError(
            "Invalid Mint event data length".to_string(),
        ));
    }
    let amount0 = U256::from_big_endian(&log.data[0..32]);
    let amount1 = U256::from_big_endian(&log.data[32..64]);
    Ok(MintEvent {
        pair_address: log.address,
        sender,
        amount0,
        amount1,
        block_number: log.block_number.unwrap().as_u64(),
        transaction_hash: log.transaction_hash.unwrap(),
        log_index: log.log_index.unwrap().as_u64(),
    })
}

pub fn parse_burn_log(log: &ethers::types::Log) -> Result<BurnEvent, EvmError> {
    if log.topics.len() < 3 {
        return Err(EvmError::ContractError(
            "Invalid Burn event log".to_string(),
        ));
    }
    let sender = Address::from_slice(&log.topics[1].as_bytes()[12..]);
    let to = Address::from_slice(&log.topics[2].as_bytes()[12..]);
    if log.data.len() < 64 {
        return Err(EvmError::ContractError(
            "Invalid Burn event data length".to_string(),
        ));
    }
    let amount0 = U256::from_big_endian(&log.data[0..32]);
    let amount1 = U256::from_big_endian(&log.data[32..64]);
    Ok(BurnEvent {
        pair_address: log.address,
        sender,
        to,
        amount0,
        amount1,
        block_number: log.block_number.unwrap().as_u64(),
        transaction_hash: log.transaction_hash.unwrap(),
        log_index: log.log_index.unwrap().as_u64(),
    })
}

pub fn cal_swap_volume(
    event: &SwapEvent,
    token0_price: Option<f64>,
    token1_price: Option<f64>,
) -> f64 {
    let mut volume = 0.0;
    if let Some(price) = token0_price {
        if !event.amount0_in.is_zero() {
            volume += event.amount0_in.as_u128() as f64 * price;
        }
        if !event.amount0_out.is_zero() {
            volume += event.amount0_out.as_u128() as f64 * price;
        }
    }
    if let Some(price) = token1_price {
        if !event.amount1_in.is_zero() {
            volume += event.amount1_in.as_u128() as f64 * price;
        }
        if !event.amount1_out.is_zero() {
            volume += event.amount1_out.as_u128() as f64 * price;
        }
    }
    volume
}

/// address tool module
pub mod address {
    use ethers::types::Address;
    use std::str::FromStr;

    /// Convert string to Address
    pub fn str_to_address(address_str: &str) -> Result<Address, String> {
        Address::from_str(address_str.trim())
            .map_err(|e| format!("Invalid Ethereum address: {}", e))
    }
}

/// Calculate the V4 price from sqrtPriceX96
pub fn cal_price_from_sqrt_price_x96(sqrt_price_x96: U256) -> f64 {
    let price_x96 = sqrt_price_x96 * sqrt_price_x96;
    (price_x96.as_u128() as f64) / (2.0_f64.powi(192))
}

/// V4 Event Analysis Tool
pub mod event_parsers {
    use super::*;
    use crate::types::{V4LiquidityModifiedEvent, V4PoolInitializedEvent, V4SwapEvent};
    pub fn parse_v4_swap_log(log: &ethers::types::Log) -> Result<V4SwapEvent, EvmError> {
        if log.topics.len() < 3 {
            return Err(EvmError::ContractError(
                "Invalid V4 Swap event log".to_string(),
            ));
        }
        let sender = Address::from_slice(&log.topics[2].as_bytes()[12..]);
        let data = &log.data;
        if data.len() < 160 {
            return Err(EvmError::ContractError(
                "Invalid V4 Swap event data length".to_string(),
            ));
        }
        let amount0 = i128::from_be_bytes(data[0..16].try_into().unwrap());
        let amount1 = i128::from_be_bytes(data[16..32].try_into().unwrap());
        let sqrt_price_x96 = U256::from_big_endian(&data[32..64]);
        let liquidity = U256::from_big_endian(&data[64..96]);
        let tick = i32::from_be_bytes(data[96..100].try_into().unwrap());
        Ok(V4SwapEvent {
            pool_address: log.address,
            sender,
            amount0,
            amount1,
            sqrt_price_x96,
            liquidity,
            tick,
            block_number: log.block_number.unwrap().as_u64(),
            transaction_hash: log.transaction_hash.unwrap(),
            log_index: log.log_index.unwrap().as_u64(),
        })
    }

    pub fn parse_v4_liquidity_modified_log(
        log: &ethers::types::Log,
    ) -> Result<V4LiquidityModifiedEvent, EvmError> {
        if log.topics.len() < 3 {
            return Err(EvmError::ContractError(
                "Invalid V4 LiquidityModified event log".to_string(),
            ));
        }
        let owner = Address::from_slice(&log.topics[2].as_bytes()[12..]);
        let data = &log.data;
        if data.len() < 72 {
            return Err(EvmError::ContractError(
                "Invalid V4 LiquidityModified event data length".to_string(),
            ));
        }
        let tick_lower = i32::from_be_bytes(data[0..4].try_into().unwrap());
        let tick_upper = i32::from_be_bytes(data[4..8].try_into().unwrap());
        let liquidity_delta = i128::from_be_bytes(data[8..24].try_into().unwrap());
        let amount0 = i128::from_be_bytes(data[24..40].try_into().unwrap());
        let amount1 = i128::from_be_bytes(data[40..56].try_into().unwrap());
        Ok(V4LiquidityModifiedEvent {
            pool_address: log.address,
            owner,
            tick_lower,
            tick_upper,
            liquidity_delta,
            amount0,
            amount1,
            block_number: log.block_number.unwrap().as_u64(),
            transaction_hash: log.transaction_hash.unwrap(),
            log_index: log.log_index.unwrap().as_u64(),
        })
    }

    pub fn parse_v4_pool_initialized_log(
        log: &ethers::types::Log,
    ) -> Result<V4PoolInitializedEvent, EvmError> {
        if log.topics.len() < 4 {
            return Err(EvmError::ContractError(
                "Invalid V4 PoolInitialized event log".to_string(),
            ));
        }
        let currency0 = Address::from_slice(&log.topics[1].as_bytes()[12..]);
        let currency1 = Address::from_slice(&log.topics[2].as_bytes()[12..]);
        let hooks = Address::from_slice(&log.topics[3].as_bytes()[12..]);
        let data = &log.data;
        if data.len() < 12 {
            return Err(EvmError::ContractError(
                "Invalid V4 PoolInitialized event data length".to_string(),
            ));
        }
        let fee = u32::from_be_bytes(data[0..4].try_into().unwrap());
        let tick_spacing = i32::from_be_bytes(data[4..8].try_into().unwrap());
        Ok(V4PoolInitializedEvent {
            pool_address: log.address,
            currency0,
            currency1,
            fee,
            tick_spacing,
            hooks,
            block_number: log.block_number.unwrap().as_u64(),
            transaction_hash: log.transaction_hash.unwrap(),
            log_index: log.log_index.unwrap().as_u64(),
        })
    }
}
