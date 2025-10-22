use crate::block::{Block, Transaction};
use crate::consensus::{ConsensusType, ProofOfStake, DelegatedProofOfStake};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// 区块链结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: u32,
    pub mining_reward: u64,
    pub balances: HashMap<String, u64>,
    pub consensus_type: ConsensusType,
    pub pos_consensus: Option<ProofOfStake>,
    pub dpos_consensus: Option<DelegatedProofOfStake>,
}

impl Blockchain {
    /// 创建新区块链
    pub fn new(difficulty: u32, mining_reward: u64) -> Self {
        let mut blockchain = Self {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            difficulty,
            mining_reward,
            balances: HashMap::new(),
            consensus_type: ConsensusType::PoW,
            pos_consensus: None,
            dpos_consensus: None,
        };

        // 创建创世区块
        let genesis_block = Block::create_genesis(difficulty);
        blockchain.chain.push(genesis_block);

        // 添加初始余额给系统账户（用于演示）
        blockchain.balances.insert("system".to_string(), 1000);

        blockchain
    }

    /// 获取最新区块
    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    /// 获取区块链长度
    pub fn get_length(&self) -> usize {
        self.chain.len()
    }

    /// 添加交易到待处理队列
    pub fn add_transaction(&mut self, mut transaction: Transaction) -> Result<(), String> {
        // 验证交易
        if transaction.sender == transaction.receiver {
            return Err("发送者和接收者不能是同一个人".to_string());
        }

        if transaction.amount == 0 {
            return Err("交易金额必须大于0".to_string());
        }

        // 检查发送者余额
        let sender_balance = self.get_balance(&transaction.sender);
        if sender_balance < transaction.amount {
            return Err(format!(
                "发送者余额不足。当前余额: {}, 交易金额: {}",
                sender_balance, transaction.amount
            ));
        }

        // 计算交易哈希
        transaction.id = transaction.calculate_hash();

        self.pending_transactions.push(transaction);
        Ok(())
    }

    /// 挖矿 - 创建新区块
    pub fn mine_pending_transactions(&mut self, miner_address: String) -> Result<Block, String> {
        if self.pending_transactions.is_empty() {
            return Err("没有待处理的交易".to_string());
        }

        // 创建矿工奖励交易
        let reward_transaction = Transaction::new(
            "system".to_string(),
            miner_address,
            self.mining_reward,
        );

        // 合并待处理交易和奖励交易
        let mut transactions = self.pending_transactions.clone();
        transactions.push(reward_transaction);

        // 创建新区块
        let prev_hash = self.get_latest_block().header.hash.clone();
        let height = self.get_length() as u64;

        let mut new_block = Block::new(
            prev_hash,
            transactions,
            height,
            self.difficulty,
        );

        // 挖矿
        new_block.mine();

        // 添加区块到链
        self.chain.push(new_block.clone());

        // 更新余额
        self.update_balances();

        // 调整难度
        self.adjust_difficulty();

        // 清空待处理交易
        self.pending_transactions.clear();

        Ok(new_block)
    }

    /// 更新账户余额
    fn update_balances(&mut self) {
        // 不清空余额，从当前余额开始更新
        for block in &self.chain {
            for transaction in &block.transactions {
                if transaction.sender != "system" {
                    let sender_balance = self.balances.get(&transaction.sender).unwrap_or(&0);
                    self.balances.insert(
                        transaction.sender.clone(),
                        sender_balance - transaction.amount,
                    );
                }

                if transaction.receiver != "genesis" {
                    let receiver_balance = self.balances.get(&transaction.receiver).unwrap_or(&0);
                    self.balances.insert(
                        transaction.receiver.clone(),
                        receiver_balance + transaction.amount,
                    );
                }
            }
        }
    }

    /// 获取账户余额
    pub fn get_balance(&self, address: &str) -> u64 {
        self.balances.get(address).unwrap_or(&0).clone()
    }

    /// 验证区块链完整性
    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            // 验证当前区块
            if !current_block.is_valid(&previous_block.header.hash) {
                return false;
            }

            // 验证哈希链
            if current_block.header.prev_hash != previous_block.header.hash {
                return false;
            }
        }
        true
    }

    /// 调整挖矿难度（基于区块生成时间）
    pub fn adjust_difficulty(&mut self) {
        if self.chain.len() < 2 {
            return; // 创世区块后第一个区块不调整
        }

        let last_block = self.chain.last().unwrap();
        let second_last_block = &self.chain[self.chain.len() - 2];

        let time_diff = last_block.header.timestamp - second_last_block.header.timestamp;
        let target_time = chrono::Duration::seconds(10); // 目标区块时间10秒

        if time_diff < target_time {
            self.difficulty = (self.difficulty + 1).min(20); // 增加难度，上限20
        } else if time_diff > target_time * 2 {
            self.difficulty = self.difficulty.saturating_sub(1).max(1); // 减少难度，下限1
        }
    }

    /// 共识机制：验证并替换为更长的有效链
    #[allow(dead_code)]
    pub fn replace_chain(&mut self, new_chain: Vec<Block>) -> bool {
        if new_chain.len() <= self.chain.len() {
            return false; // 新链不更长
        }

        // 创建临时区块链验证新链
        let temp_blockchain = Blockchain {
            chain: new_chain,
            pending_transactions: Vec::new(),
            difficulty: self.difficulty,
            mining_reward: self.mining_reward,
            balances: HashMap::new(),
            consensus_type: self.consensus_type.clone(),
            pos_consensus: self.pos_consensus.clone(),
            dpos_consensus: self.dpos_consensus.clone(),
        };

        if temp_blockchain.is_chain_valid() {
            self.chain = temp_blockchain.chain;
            self.update_balances();
            self.adjust_difficulty(); // 基于新链调整难度
            return true;
        }
        false
    }

    /// 获取区块链的总交易数
    pub fn get_total_transactions(&self) -> usize {
        self.chain.iter().map(|block| block.transactions.len()).sum()
    }

    /// 打印区块链信息
    pub fn print_chain(&self) {
        println!("\n=== 区块链信息 ===");
        println!("区块链长度: {}", self.get_length());
        println!("总交易数: {}", self.get_total_transactions());
        println!("挖矿难度: {}", self.difficulty);
        println!("挖矿奖励: {}", self.mining_reward);
        println!("待处理交易: {}", self.pending_transactions.len());
        println!("区块链有效性: {}", self.is_chain_valid());

        println!("\n=== 区块列表 ===");
        for (i, block) in self.chain.iter().enumerate() {
            println!("{}: {}", i, block);
        }

        println!("\n=== 账户余额 ===");
        for (address, balance) in &self.balances {
            if *balance > 0 {
                println!("{}: {}", address, balance);
            }
        }
    }

    /// 保存区块链到文件
    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(filename, json)?;
        println!("区块链已保存到文件: {}", filename);
        Ok(())
    }

    /// 从文件加载区块链
    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !fs::metadata(filename).is_ok() {
            return Err(format!("文件不存在: {}", filename).into());
        }
        let json = fs::read_to_string(filename)?;
        let mut blockchain: Blockchain = serde_json::from_str(&json)?;
        println!("区块链已从文件加载: {}", filename);

        // 如果加载的区块链没有余额，添加初始余额给系统账户
        if blockchain.balances.is_empty() {
            blockchain.balances.insert("system".to_string(), 1000);
        }

        Ok(blockchain)
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new(4, 100) // 默认难度4，奖励100
    }
}
