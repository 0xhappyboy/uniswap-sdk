use crate::abi::{IFarmFactory, IMasterChef, IStakingRewards};
use crate::types::*;
use crate::{EvmClient, EvmError};
use ethers::providers::Provider;
use ethers::types::{Address, H256, U256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct FarmService {
    client: Arc<EvmClient>,
    farm_cache: Arc<Mutex<HashMap<Address, FarmPool>>>,
}

impl FarmService {
    pub fn new(client: Arc<EvmClient>) -> Self {
        Self {
            client,
            farm_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Fetches all farm pools from the factory contract
    ///
    /// # Arguments
    /// factory_address - Address of the farm factory contract
    /// chain - EVM chain type
    ///
    /// Returns
    /// Vector of farm pools sorted by APR in descending order
    ///
    /// Example
    /// ```rust
    /// let farms = farm_service.get_all_farms(factory_address, EvmType::Mainnet).await?;
    /// for farm in farms {
    ///     println!("Farm: {}, APR: {:.2}%", farm.farm_address, farm.apr);
    /// }
    /// ```
    pub async fn get_all_farms(
        &self,
        factory_address: Address,
        chain: crate::EvmType,
    ) -> Result<Vec<FarmPool>, EvmError> {
        let factory = self.farm_factory(factory_address);
        let farm_count = factory
            .all_farms_length()
            .call()
            .await
            .map_err(|e| EvmError::ContractError(format!("Failed to get farm count: {}", e)))?;
        let mut farms = Vec::new();
        for i in 0..farm_count.as_u64() {
            if let Ok(farm_address) = factory.all_farms(U256::from(i)).call().await {
                if let Ok(farm_detail) = self.get_farm_detail(farm_address).await {
                    farms.push(farm_detail);
                }
            }
        }
        // apr sort
        farms.sort_by(|a, b| {
            b.apr
                .partial_cmp(&a.apr)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(farms)
    }

    /// Retrieves detailed information about a specific farm
    ///
    /// # Arguments
    /// * `farm_address` - Address of the farm contract
    ///
    /// # Returns
    /// FarmPool struct containing all farm details
    ///
    /// # Example
    /// ```rust
    /// let farm_detail = farm_service.get_farm_detail(farm_address).await?;
    /// println!("Total staked: {}, TVL: ${:.2}", farm_detail.total_staked, farm_detail.tvl);
    /// ```
    pub async fn get_farm_detail(&self, farm_address: Address) -> Result<FarmPool, EvmError> {
        {
            let cache = self.farm_cache.lock().await;
            if let Some(cached) = cache.get(&farm_address) {
                return Ok(cached.clone());
            }
        }
        let farm = self.staking_rewards(farm_address);
        let total_staked =
            farm.total_supply().call().await.map_err(|e| {
                EvmError::ContractError(format!("Failed to get total staked: {}", e))
            })?;
        let reward_rate = farm.reward_rate().call().await.unwrap_or(U256::zero());
        let period_finish = farm.period_finish().call().await.unwrap_or(U256::zero());
        let rewards_duration = farm
            .rewards_duration()
            .call()
            .await
            .unwrap_or(U256::from(7 * 24 * 3600));
        let staking_token =
            farm.staking_token().call().await.map_err(|e| {
                EvmError::ContractError(format!("Failed to get staking token: {}", e))
            })?;
        let reward_token =
            farm.rewards_token().call().await.map_err(|e| {
                EvmError::ContractError(format!("Failed to get reward token: {}", e))
            })?;
        let apr = self
            .calculate_apr(total_staked, reward_rate, rewards_duration)
            .await?;
        let tvl = self.calculate_tvl(staking_token, total_staked).await?;
        let farm_pool = FarmPool {
            farm_address,
            staking_token,
            reward_token,
            staking_token_symbol: "LP".to_string(),    // TODO
            reward_token_symbol: "REWARD".to_string(), // TODO
            total_staked,
            reward_rate,
            rewards_duration: rewards_duration.as_u64(),
            period_finish: period_finish.as_u64(),
            last_update_time: 0,
            reward_per_token_stored: U256::zero(),
            apr,
            tvl,
            is_active: period_finish.as_u64() > chrono::Utc::now().timestamp() as u64,
            created_block: 0,
        };
        {
            let mut cache = self.farm_cache.lock().await;
            cache.insert(farm_address, farm_pool.clone());
        }
        Ok(farm_pool)
    }

    /// Gets user's position in a farm including staked amount and pending rewards
    ///
    /// # Arguments
    /// farm_address - Address of the farm contract
    /// user_address - Address of the user
    ///
    /// Example
    /// ```rust
    /// let position = farm_service.get_user_position(farm_address, user_address).await?;
    /// println!("Staked: {}, Pending rewards: {}", position.staked_amount, position.pending_rewards);
    /// ```
    pub async fn get_user_position(
        &self,
        farm_address: Address,
        user_address: Address,
    ) -> Result<UserFarmPosition, EvmError> {
        let farm = self.staking_rewards(farm_address);
        let staked_amount =
            farm.balance_of(user_address).call().await.map_err(|e| {
                EvmError::ContractError(format!("Failed to get user balance: {}", e))
            })?;
        let pending_rewards = farm.earned(user_address).call().await.map_err(|e| {
            EvmError::ContractError(format!("Failed to get pending rewards: {}", e))
        })?;
        Ok(UserFarmPosition {
            user_address,
            farm_address,
            staked_amount,
            pending_rewards,
            reward_debt: U256::zero(),
            last_update_time: chrono::Utc::now().timestamp() as u64,
            total_earned: U256::zero(),
        })
    }

    /// Stakes LP tokens into a farm
    ///
    /// # Arguments
    /// farm_address` - Address of the farm contract
    /// amount` - Amount of LP tokens to stake
    ///
    /// Example
    /// ```rust
    /// let amount = U256::from(1000000000000000000u64); // 1.0 LP token
    /// let tx_hash = farm_service.stake(farm_address, amount).await?;
    /// println!("Stake transaction: {:?}", tx_hash);
    /// ```
    pub async fn stake(&self, farm_address: Address, amount: U256) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let farm = self.staking_rewards(farm_address);
        let tx = farm.stake(amount);
        let pending_tx = tx
            .send()
            .await
            .map_err(|e| EvmError::TransactionError(format!("Failed to stake: {}", e)))?;
        Ok(pending_tx.tx_hash())
    }

    /// Unstakes LP tokens from a farm
    ///
    /// # Arguments
    /// farm_address - Address of the farm contract
    /// amount - Amount of LP tokens to unstake
    ///
    /// Example
    /// ```rust
    /// let amount = U256::from(500000000000000000u64); // 0.5 LP token
    /// let tx_hash = farm_service.unstake(farm_address, amount).await?;
    /// println!("Unstake transaction: {:?}", tx_hash);
    /// ```
    pub async fn unstake(&self, farm_address: Address, amount: U256) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let farm = self.staking_rewards(farm_address);
        let tx = farm.withdraw(amount);
        let pending_tx = tx
            .send()
            .await
            .map_err(|e| EvmError::TransactionError(format!("Failed to unstake: {}", e)))?;
        Ok(pending_tx.tx_hash())
    }

    /// Claims pending rewards from a farm
    ///
    /// # Arguments
    /// farm_address - Address of the farm contract
    ///
    /// Example
    /// ```rust
    /// let tx_hash = farm_service.claim_rewards(farm_address).await?;
    /// println!("Claim rewards transaction: {:?}", tx_hash);
    /// ```
    pub async fn claim_rewards(&self, farm_address: Address) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let farm = self.staking_rewards(farm_address);
        let tx = farm.get_reward();
        let pending_tx = tx
            .send()
            .await
            .map_err(|e| EvmError::TransactionError(format!("Failed to claim rewards: {}", e)))?;

        Ok(pending_tx.tx_hash())
    }

    /// Exits farm by unstaking all tokens and claiming rewards in one transaction
    ///
    /// # Arguments
    /// farm_address - Address of the farm contract
    ///
    /// # Example
    /// ```rust
    /// let tx_hash = farm_service.exit_farm(farm_address).await?;
    /// println!("Exit farm transaction: {:?}", tx_hash);
    /// ```
    pub async fn exit_farm(&self, farm_address: Address) -> Result<H256, EvmError> {
        if self.client.wallet.is_none() {
            return Err(EvmError::WalletError("No wallet configured".to_string()));
        }
        let farm = self.staking_rewards(farm_address);
        let tx = farm.exit();
        let pending_tx = tx
            .send()
            .await
            .map_err(|e| EvmError::TransactionError(format!("Failed to exit farm: {}", e)))?;
        Ok(pending_tx.tx_hash())
    }

    /// Calculates Annual Percentage Rate for a farm
    async fn calculate_apr(
        &self,
        total_staked: U256,
        reward_rate: U256,
        rewards_duration: U256,
    ) -> Result<f64, EvmError> {
        if total_staked.is_zero() || reward_rate.is_zero() {
            return Ok(0.0);
        }
        let annual_rewards = reward_rate * U256::from(365 * 24 * 3600);
        let apr = (annual_rewards.as_u128() as f64) / (total_staked.as_u128() as f64) * 100.0;
        Ok(apr)
    }

    /// Calculates Total Value Locked for a farm
    async fn calculate_tvl(&self, lp_token: Address, total_staked: U256) -> Result<f64, EvmError> {
        todo!();
        Ok(total_staked.as_u128() as f64 / 1e18)
    }

    /// Gets APY data for a farm including 7-day and 30-day projections
    ///
    /// # Arguments
    /// farm_address - Address of the farm contract
    ///
    /// # Example
    /// ```rust
    /// let apy_data = farm_service.get_farm_apy(farm_address).await?;
    /// println!("7-day APY: {:.2}%, Daily rewards: {:.2}", apy_data.apy_7d, apy_data.daily_rewards);
    /// ```
    pub async fn get_farm_apy(&self, farm_address: Address) -> Result<FarmAPY, EvmError> {
        let farm_detail = self.get_farm_detail(farm_address).await?;
        let apr_7d = farm_detail.apr;
        let apr_30d = farm_detail.apr * 0.9;
        // APY = (1 + APR/n)^n - 1, where n = 365 (daily compound interest)
        let apy_7d = (1.0 + apr_7d / 100.0 / 365.0).powf(365.0) - 1.0;
        let apy_30d = (1.0 + apr_30d / 100.0 / 365.0).powf(365.0) - 1.0;
        let daily_rewards = (farm_detail.reward_rate.as_u128() as f64 * 24.0 * 3600.0) / 1e18;
        let weekly_rewards = daily_rewards * 7.0;
        Ok(FarmAPY {
            farm_address,
            apr_7d,
            apr_30d,
            apy_7d: apy_7d * 100.0,
            apy_30d: apy_30d * 100.0,
            daily_rewards,
            weekly_rewards,
        })
    }

    /// Retrieves historical staking data for a farm
    ///
    /// # Arguments
    /// farm_address - Address of the farm contract
    /// days - Number of days of historical data to retrieve
    ///
    /// # Example
    /// ```rust
    /// let history = farm_service.get_farm_history(farm_address, 30).await?;
    /// for day in history {
    ///     println!("Date: {}, Total staked: {}", day.timestamp, day.total_staked);
    /// }
    /// ```
    pub async fn get_farm_history(
        &self,
        farm_address: Address,
        days: u64,
    ) -> Result<Vec<FarmStakingHistory>, EvmError> {
        todo!();
        let farm_detail = self.get_farm_detail(farm_address).await?;
        let mut history = Vec::new();
        let now = chrono::Utc::now().timestamp() as u64;
        for i in 0..days {
            let timestamp = now - (i * 24 * 3600);
            history.push(FarmStakingHistory {
                timestamp,
                total_staked: farm_detail.total_staked * (100 - i as u32 * 2) / 100, // 模拟变化
                reward_rate: farm_detail.reward_rate,
                apr: farm_detail.apr * (1.0 - i as f64 * 0.01),
            });
        }
        history.reverse();
        Ok(history)
    }

    fn staking_rewards(
        &self,
        farm_address: Address,
    ) -> IStakingRewards<Provider<ethers::providers::Http>> {
        IStakingRewards::new(farm_address, self.client.provider.clone())
    }

    fn farm_factory(
        &self,
        factory_address: Address,
    ) -> IFarmFactory<Provider<ethers::providers::Http>> {
        IFarmFactory::new(factory_address, self.client.provider.clone())
    }
}
