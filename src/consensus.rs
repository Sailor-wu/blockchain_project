use crate::block::{Block, Transaction};
use crate::blockchain::Blockchain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 共识算法类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusType {
    PoW,  // 工作量证明
    PoS,  // 权益证明
    DPoS, // 委托权益证明
}

/// 共识算法 trait
pub trait Consensus {
    /// 验证区块
    fn validate_block(&self, block: &Block, previous_block: &Block) -> bool;

    /// 选择验证者
    fn select_validator(&self, blockchain: &Blockchain) -> Option<String>;

    /// 计算验证者权重
    fn calculate_validator_weight(&self, blockchain: &Blockchain, validator: &str) -> u64;

    /// 验证交易
    fn validate_transaction(&self, transaction: &Transaction, blockchain: &Blockchain) -> bool;

    /// 获取共识类型
    fn get_type(&self) -> ConsensusType;
}

/// 质押信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub amount: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub validator: String,
}

/// 委托信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationInfo {
    pub delegator: String,
    pub amount: u64,
    pub candidate: String,
}

/// PoS 共识实现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfStake {
    pub stakes: HashMap<String, StakeInfo>,
    pub minimum_stake: u64,
}

impl ProofOfStake {
    pub fn new(minimum_stake: u64) -> Self {
        Self {
            stakes: HashMap::new(),
            minimum_stake,
        }
    }

    /// 质押代币
    pub fn stake(&mut self, validator: String, amount: u64) -> Result<(), String> {
        if amount < self.minimum_stake {
            return Err(format!("质押金额不足，最小需要 {}", self.minimum_stake));
        }

        let stake_info = StakeInfo {
            amount,
            start_time: chrono::Utc::now(),
            validator: validator.clone(),
        };

        self.stakes.insert(validator, stake_info);
        Ok(())
    }

    /// 取消质押
    pub fn unstake(&mut self, validator: String) -> Result<(), String> {
        if let Some(_) = self.stakes.remove(&validator) {
            Ok(())
        } else {
            Err("未找到质押信息".to_string())
        }
    }
}

impl Consensus for ProofOfStake {
    fn validate_block(&self, block: &Block, previous_block: &Block) -> bool {
        // 验证区块哈希
        if !block.is_valid(&previous_block.header.hash) {
            return false;
        }

        // 验证验证者
        if let Some(validator) = &block.header.validator {
            if !self.stakes.contains_key(validator) {
                return false;
            }
        }

        true
    }

    fn select_validator(&self, _blockchain: &Blockchain) -> Option<String> {
        if self.stakes.is_empty() {
            return None;
        }

        // 简单的随机选择（实际应该使用更复杂的算法）
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let validators: Vec<String> = self.stakes.keys().cloned().collect();
        let index = rng.gen_range(0..validators.len());
        Some(validators[index].clone())
    }

    fn calculate_validator_weight(&self, _blockchain: &Blockchain, validator: &str) -> u64 {
        self.stakes.get(validator).map(|s| s.amount).unwrap_or(0)
    }

    fn validate_transaction(&self, transaction: &Transaction, blockchain: &Blockchain) -> bool {
        // 基本交易验证
        if transaction.amount == 0 {
            return false;
        }

        // 检查发送者余额
        let sender_balance = blockchain.get_balance(&transaction.sender);
        sender_balance >= transaction.amount
    }

    fn get_type(&self) -> ConsensusType {
        ConsensusType::PoS
    }
}

/// DPoS 共识实现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedProofOfStake {
    pub stakes: HashMap<String, StakeInfo>,
    pub delegations: HashMap<String, DelegationInfo>,
    pub candidates: Vec<String>,
    pub minimum_stake: u64,
    pub minimum_delegation: u64,
}

impl DelegatedProofOfStake {
    pub fn new(minimum_stake: u64, minimum_delegation: u64) -> Self {
        Self {
            stakes: HashMap::new(),
            delegations: HashMap::new(),
            candidates: Vec::new(),
            minimum_stake,
            minimum_delegation,
        }
    }

    /// 注册候选人
    pub fn register_candidate(&mut self, candidate: String, amount: u64) -> Result<(), String> {
        if amount < self.minimum_stake {
            return Err(format!("候选人质押金额不足，最小需要 {}", self.minimum_stake));
        }

        let stake_info = StakeInfo {
            amount,
            start_time: chrono::Utc::now(),
            validator: candidate.clone(),
        };

        self.stakes.insert(candidate.clone(), stake_info);
        self.candidates.push(candidate);
        Ok(())
    }

    /// 委托投票
    pub fn delegate(&mut self, delegator: String, candidate: String, amount: u64) -> Result<(), String> {
        if amount < self.minimum_delegation {
            return Err(format!("委托金额不足，最小需要 {}", self.minimum_delegation));
        }

        let delegation_info = DelegationInfo {
            delegator: delegator.clone(),
            amount,
            candidate: candidate.clone(),
        };

        let key = format!("{}:{}", delegator, candidate);
        self.delegations.insert(key, delegation_info);
        Ok(())
    }

    /// 计算候选人总权重（自有质押 + 委托）
    pub fn calculate_candidate_weight(&self, candidate: &str) -> u64 {
        let own_stake = self.stakes.get(candidate).map(|s| s.amount).unwrap_or(0);

        let delegated_amount: u64 = self.delegations
            .iter()
            .filter(|(_, d)| d.candidate == candidate)
            .map(|(_, d)| d.amount)
            .sum();

        own_stake + delegated_amount
    }
}

impl Consensus for DelegatedProofOfStake {
    fn validate_block(&self, block: &Block, previous_block: &Block) -> bool {
        // 验证区块哈希
        if !block.is_valid(&previous_block.header.hash) {
            return false;
        }

        // 验证验证者是否为候选人
        if let Some(validator) = &block.header.validator {
            if !self.candidates.contains(&validator) {
                return false;
            }
        }

        true
    }

    fn select_validator(&self, blockchain: &Blockchain) -> Option<String> {
        if self.candidates.is_empty() {
            return None;
        }

        // 选择权重最高的候选人
        let mut max_weight = 0;
        let mut selected_validator = None;

        for candidate in &self.candidates {
            let weight = self.calculate_candidate_weight(candidate);
            if weight > max_weight {
                max_weight = weight;
                selected_validator = Some(candidate.clone());
            }
        }

        selected_validator
    }

    fn calculate_validator_weight(&self, _blockchain: &Blockchain, validator: &str) -> u64 {
        self.calculate_candidate_weight(validator)
    }

    fn validate_transaction(&self, transaction: &Transaction, blockchain: &Blockchain) -> bool {
        // 基本交易验证
        if transaction.amount == 0 {
            return false;
        }

        // 检查发送者余额
        let sender_balance = blockchain.get_balance(&transaction.sender);
        sender_balance >= transaction.amount
    }

    fn get_type(&self) -> ConsensusType {
        ConsensusType::DPoS
    }
}
