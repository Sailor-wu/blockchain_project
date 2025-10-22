use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{self, Display};

/// 区块头信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub timestamp: DateTime<Utc>,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: u32,
    pub validator: Option<String>, // 验证者地址（用于 PoS/DPoS）
}

/// 区块数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub height: u64,
}

/// 交易结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
}

impl Transaction {
    /// 创建新交易
    pub fn new(sender: String, receiver: String, amount: u64) -> Self {
        let id = format!("tx_{}", Utc::now().timestamp());
        Self {
            id,
            sender,
            receiver,
            amount,
            timestamp: Utc::now(),
        }
    }

    /// 计算交易哈希
    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}",
            self.sender,
            self.receiver,
            self.amount,
            self.timestamp.timestamp(),
            self.id
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl Block {
    /// 创建新区块
    pub fn new(
        prev_hash: String,
        transactions: Vec<Transaction>,
        height: u64,
        difficulty: u32,
    ) -> Self {
        let timestamp = Utc::now();
        let mut block = Self {
            header: BlockHeader {
                timestamp,
                prev_hash,
                hash: String::new(),
                nonce: 0,
                difficulty,
                validator: None,
            },
            transactions,
            height,
        };

        block.header.hash = block.calculate_hash();
        block
    }

    /// 创建创世区块
    pub fn create_genesis(difficulty: u32) -> Self {
        let genesis_transaction = Transaction::new(
            "system".to_string(),
            "genesis".to_string(),
            0,
        );
        Self::new(
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            vec![genesis_transaction],
            0,
            difficulty,
        )
    }

    /// 计算区块哈希
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // 构建要哈希的数据
        let data = format!(
            "{}{}{}{}{}{}",
            self.header.timestamp.timestamp(),
            self.header.prev_hash,
            self.header.nonce,
            self.header.difficulty,
            self.height,
            self.transactions
                .iter()
                .map(|tx| tx.calculate_hash())
                .collect::<String>()
        );

        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 挖矿 - 寻找合适的nonce值
    pub fn mine(&mut self) {
        println!("Mining block {}...", self.height);

        while !self.is_valid_hash() {
            self.header.nonce += 1;
            self.header.hash = self.calculate_hash();
        }

        println!("Block {} mined! Nonce: {}", self.height, self.header.nonce);
    }

    /// 验证哈希是否满足难度要求
    pub fn is_valid_hash(&self) -> bool {
        let prefix = "0".repeat(self.header.difficulty as usize);
        self.header.hash.starts_with(&prefix)
    }

    /// 验证区块的有效性
    pub fn is_valid(&self, prev_hash: &str) -> bool {
        // 验证前区块哈希
        if self.header.prev_hash != prev_hash {
            return false;
        }

        // 验证当前哈希
        if self.header.hash != self.calculate_hash() {
            return false;
        }

        // 验证难度要求
        if !self.is_valid_hash() {
            return false;
        }

        true
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block #{} - Hash: {} - Prev: {} - Nonce: {} - Transactions: {}",
            self.height,
            self.header.hash,
            self.header.prev_hash,
            self.header.nonce,
            self.transactions.len()
        )
    }
}
