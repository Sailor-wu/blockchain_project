use crate::blockchain::Blockchain;
use crate::block::{Block, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{Read, Write};
use bincode::{serialize, deserialize};

/// P2P 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// 新交易
    NewTransaction(Transaction),
    /// 新区块
    NewBlock(Block),
    /// 请求区块链
    RequestChain,
    /// 响应区块链
    ResponseChain(Vec<Block>),
    /// 请求区块链长度
    RequestChainLength,
    /// 响应区块链长度
    ResponseChainLength(usize),
    /// 请求特定区块范围
    RequestBlocks { start: u64, end: u64 },
    /// 响应区块范围
    ResponseBlocks(Vec<Block>),
    /// 节点发现
    Discovery(SocketAddr),
    /// 心跳消息
    Ping,
    /// 心跳响应
    Pong,
    /// 节点状态同步
    SyncStatus {
        chain_length: usize,
        latest_hash: String,
        total_transactions: usize,
    },
    /// 同步完成确认
    SyncComplete,
}

/// P2P 节点
pub struct P2PNode {
    pub address: SocketAddr,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub peers: Arc<Mutex<HashMap<SocketAddr, PeerInfo>>>,
    pub listener: Option<TcpListener>,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub last_seen: std::time::Instant,
}

impl P2PNode {
    /// 创建新节点
    pub fn new(address: SocketAddr, blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            address,
            blockchain,
            peers: Arc::new(Mutex::new(HashMap::new())),
            listener: None,
        }
    }

    /// 启动节点
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 启动 P2P 节点: {}", self.address);

        // 绑定监听器
        let listener = TcpListener::bind(self.address)?;
        self.listener = Some(listener.try_clone()?);

        println!("✅ P2P 节点监听在: {}", self.address);

        // 启动监听线程
        let peers = self.peers.clone();
        let blockchain = self.blockchain.clone();
        let listener_clone = listener.try_clone()?;

        thread::spawn(move || {
            Self::listen_for_connections(listener_clone, peers, blockchain);
        });

        // 启动心跳线程
        let peers_heartbeat = self.peers.clone();
        thread::spawn(move || {
            Self::heartbeat_loop(peers_heartbeat);
        });

        Ok(())
    }

    /// 连接到其他节点
    pub fn connect_to_peer(&self, peer_address: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔗 连接到节点: {}", peer_address);

        match TcpStream::connect(peer_address) {
                Ok(mut stream) => {
                // 发送发现消息
                let discovery_msg = Message::Discovery(self.address);
                let data = serialize(&discovery_msg)?;
                stream.write_all(&data)?;

                // 添加到对等节点列表
                self.peers.lock().unwrap().insert(peer_address, PeerInfo {
                    address: peer_address,
                    last_seen: std::time::Instant::now(),
                });

                println!("✅ 成功连接到节点: {}", peer_address);
                Ok(())
            }
            Err(e) => {
                println!("❌ 连接失败 {}: {}", peer_address, e);
                Err(e.into())
            }
        }
    }

    /// 广播交易
    pub fn broadcast_transaction(&self, transaction: Transaction) -> Result<(), Box<dyn std::error::Error>> {
        let message = Message::NewTransaction(transaction);
        self.broadcast_message(message)
    }

    /// 广播区块
    pub fn broadcast_block(&self, block: Block) -> Result<(), Box<dyn std::error::Error>> {
        let message = Message::NewBlock(block);
        self.broadcast_message(message)
    }

    /// 广播消息到所有对等节点
    fn broadcast_message(&self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        let data = serialize(&message)?;
        let peers = self.peers.lock().unwrap();

        for (peer_addr, _) in peers.iter() {
            if let Err(e) = self.send_to_peer(*peer_addr, &data) {
                println!("❌ 发送消息到 {} 失败: {}", peer_addr, e);
            }
        }

        Ok(())
    }

    /// 发送消息到特定节点
    fn send_to_peer(&self, peer_address: SocketAddr, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect(peer_address)?;
        stream.write_all(data)?;
        Ok(())
    }

    /// 监听连接
    fn listen_for_connections(
        listener: TcpListener,
        peers: Arc<Mutex<HashMap<SocketAddr, PeerInfo>>>,
        blockchain: Arc<Mutex<Blockchain>>,
    ) {
        println!("👂 开始监听 P2P 连接...");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let peer_addr = stream.peer_addr().unwrap();
                    println!("🔗 新连接来自: {}", peer_addr);

                    // 添加到对等节点列表
                    peers.lock().unwrap().insert(peer_addr, PeerInfo {
                        address: peer_addr,
                        last_seen: std::time::Instant::now(),
                    });

                    // 处理消息
                    let blockchain_clone = blockchain.clone();
                    let peers_clone = peers.clone();

                    thread::spawn(move || {
                        Self::handle_connection(stream, blockchain_clone, peers_clone);
                    });
                }
                Err(e) => {
                    println!("❌ 连接错误: {}", e);
                }
            }
        }
    }

    /// 处理连接
    fn handle_connection(
        mut stream: TcpStream,
        blockchain: Arc<Mutex<Blockchain>>,
        peers: Arc<Mutex<HashMap<SocketAddr, PeerInfo>>>,
    ) {
        let peer_addr = stream.peer_addr().unwrap();
        let mut buffer = [0; 4096]; // 增加缓冲区大小以支持更大的消息

        loop {
            match stream.read(&mut buffer) {
                Ok(size) if size > 0 => {
                    let data = &buffer[..size];
                    match deserialize::<Message>(data) {
                        Ok(message) => {
                            if let Err(e) = Self::process_message(message, &blockchain, &peers, &mut stream, peer_addr) {
                                println!("❌ 处理消息失败: {}", e);
                            }
                        }
                        Err(e) => {
                            println!("❌ 反序列化消息失败: {}", e);
                        }
                    }
                }
                Ok(_) => break, // 连接关闭
                Err(e) => {
                    println!("❌ 读取消息失败: {}", e);
                    break;
                }
            }
        }
    }

    /// 处理接收到的消息
    fn process_message(
        message: Message,
        blockchain: &Arc<Mutex<Blockchain>>,
        peers: &Arc<Mutex<HashMap<SocketAddr, PeerInfo>>>,
        stream: &mut TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message {
            Message::NewTransaction(transaction) => {
                println!("📦 收到新交易: {:?}", transaction.id);
                let mut blockchain = blockchain.lock().unwrap();
                if let Err(e) = blockchain.add_transaction(transaction) {
                    println!("❌ 添加交易失败: {}", e);
                }
            }
            Message::NewBlock(block) => {
                println!("🧱 收到新区块: {}", block.header.hash);
                let mut blockchain = blockchain.lock().unwrap();
                Self::handle_new_block(&mut blockchain, block);
            }
            Message::RequestChain => {
                println!("📋 收到完整区块链请求");
                Self::handle_chain_request(blockchain, stream, peer_addr);
            }
            Message::RequestChainLength => {
                println!("📏 收到区块链长度请求");
                Self::handle_chain_length_request(blockchain, stream, peer_addr);
            }
            Message::RequestBlocks { start, end } => {
                println!("📦 收到区块范围请求: {}-{}", start, end);
                Self::handle_blocks_request(blockchain, start, end, stream, peer_addr);
            }
            Message::ResponseChain(chain) => {
                println!("📋 收到完整区块链响应，长度: {}", chain.len());
                let mut blockchain = blockchain.lock().unwrap();
                Self::handle_chain_response(&mut blockchain, chain);
            }
            Message::ResponseChainLength(length) => {
                println!("📏 收到区块链长度响应: {}", length);
                Self::handle_chain_length_response(blockchain, length, stream, peer_addr);
            }
            Message::ResponseBlocks(blocks) => {
                println!("📦 收到区块响应，数量: {}", blocks.len());
                let mut blockchain = blockchain.lock().unwrap();
                Self::handle_blocks_response(&mut blockchain, blocks);
            }
            Message::SyncStatus { chain_length, latest_hash, total_transactions } => {
                println!("🔄 收到同步状态: 链长度={}, 最新哈希={}, 总交易={}",
                         chain_length, latest_hash, total_transactions);
                Self::handle_sync_status(blockchain, chain_length, latest_hash, total_transactions);
            }
            Message::SyncComplete => {
                println!("✅ 收到同步完成确认");
            }
            Message::Discovery(peer_addr) => {
                println!("🔍 发现新节点: {}", peer_addr);
                peers.lock().unwrap().insert(peer_addr, PeerInfo {
                    address: peer_addr,
                    last_seen: std::time::Instant::now(),
                });
            }
            Message::Ping => {
                println!("🏓 收到 Ping");
                // 响应 Pong - 这里需要发送响应
            }
            Message::Pong => {
                println!("🏓 收到 Pong");
            }
        }

        Ok(())
    }

    /// 心跳循环
    fn heartbeat_loop(peers: Arc<Mutex<HashMap<SocketAddr, PeerInfo>>>) {
        loop {
            thread::sleep(std::time::Duration::from_secs(30));

            let mut peers = peers.lock().unwrap();
            let mut to_remove = Vec::new();

            for (addr, peer_info) in peers.iter() {
                if peer_info.last_seen.elapsed() > std::time::Duration::from_secs(60) {
                    println!("💔 节点 {} 超时，移除", addr);
                    to_remove.push(*addr);
                }
            }

            for addr in to_remove {
                peers.remove(&addr);
            }
        }
    }

    /// 获取对等节点列表
    pub fn get_peers(&self) -> Vec<SocketAddr> {
        self.peers.lock().unwrap().keys().cloned().collect()
    }

    /// 停止节点
    pub fn stop(&mut self) {
        println!("🛑 停止 P2P 节点");
        // 这里可以添加清理逻辑
    }

    /// 处理新区块
    fn handle_new_block(blockchain: &mut Blockchain, block: Block) {
        // 验证区块
        if !blockchain.is_chain_valid() {
            println!("❌ 区块验证失败");
            return;
        }

        // 检查是否已经有这个区块
        let latest_block = blockchain.get_latest_block();
        if block.header.prev_hash != latest_block.header.hash {
            println!("⚠️ 收到非连续区块，尝试同步");
            // 这里应该触发同步逻辑
            return;
        }

        // 尝试替换链（如果新区块更长）
        let new_chain = vec![block];
        if blockchain.replace_chain(new_chain) {
            println!("✅ 区块链已更新");
        } else {
            println!("ℹ️ 区块已存在或不是更长的链");
        }
    }

    /// 处理区块链请求
    fn handle_chain_request(blockchain: &Arc<Mutex<Blockchain>>, stream: &mut TcpStream, peer_addr: SocketAddr) {
        let blockchain = blockchain.lock().unwrap();
        let chain = blockchain.chain.clone();

        // 发送区块链响应
        let response = Message::ResponseChain(chain.clone());
        match serialize(&response) {
            Ok(data) => {
                if let Err(e) = stream.write_all(&data) {
                    println!("❌ 发送区块链响应失败: {}", e);
                } else {
                    println!("📤 发送区块链响应到 {}，长度: {}", peer_addr, chain.len());
                }
            }
            Err(e) => {
                println!("❌ 序列化区块链响应失败: {}", e);
            }
        }
    }

    /// 处理链长度请求
    fn handle_chain_length_request(blockchain: &Arc<Mutex<Blockchain>>, stream: &mut TcpStream, peer_addr: SocketAddr) {
        let blockchain = blockchain.lock().unwrap();
        let length = blockchain.get_length();

        // 发送链长度响应
        let response = Message::ResponseChainLength(length);
        match serialize(&response) {
            Ok(data) => {
                if let Err(e) = stream.write_all(&data) {
                    println!("❌ 发送链长度响应失败: {}", e);
                } else {
                    println!("📤 发送链长度响应到 {}: {}", peer_addr, length);
                }
            }
            Err(e) => {
                println!("❌ 序列化链长度响应失败: {}", e);
            }
        }
    }

    /// 处理区块范围请求
    fn handle_blocks_request(blockchain: &Arc<Mutex<Blockchain>>, start: u64, end: u64, stream: &mut TcpStream, peer_addr: SocketAddr) {
        let blockchain = blockchain.lock().unwrap();
        let chain_length = blockchain.get_length() as u64;

        if start >= chain_length || end < start {
            println!("❌ 无效的区块范围请求: {}-{}", start, end);
            return;
        }

        let actual_end = end.min(chain_length - 1);
        let blocks: Vec<Block> = blockchain.chain
            .iter()
            .skip(start as usize)
            .take((actual_end - start + 1) as usize)
            .cloned()
            .collect();

        // 发送区块范围响应
        let response = Message::ResponseBlocks(blocks);
        match serialize(&response) {
            Ok(data) => {
                if let Err(e) = stream.write_all(&data) {
                    println!("❌ 发送区块范围响应失败: {}", e);
                } else {
                    println!("📤 发送区块范围响应到 {}: {}-{} ({} 区块)",
                             peer_addr, start, actual_end, actual_end - start + 1);
                }
            }
            Err(e) => {
                println!("❌ 序列化区块范围响应失败: {}", e);
            }
        }
    }

    /// 处理链响应
    fn handle_chain_response(blockchain: &mut Blockchain, new_chain: Vec<Block>) {
        println!("🔄 处理区块链响应，长度: {}", new_chain.len());

        // 验证新链
        if new_chain.is_empty() {
            println!("❌ 收到空链");
            return;
        }

        // 验证链的完整性
        let temp_chain = new_chain.clone();
        let mut is_valid = true;

        for i in 1..temp_chain.len() {
            let current = &temp_chain[i];
            let previous = &temp_chain[i - 1];

            if !current.is_valid(&previous.header.hash) {
                println!("❌ 链验证失败在区块 {}", i);
                is_valid = false;
                break;
            }
        }

        if !is_valid {
            println!("❌ 新链验证失败，忽略");
            return;
        }

        // 比较链长度
        if new_chain.len() > blockchain.get_length() {
            println!("📈 新链更长 ({} > {})，替换区块链",
                     new_chain.len(), blockchain.get_length());

            if blockchain.replace_chain(new_chain) {
                println!("✅ 区块链替换成功");
                // 广播新链到其他节点
                // TODO: 广播新链
            } else {
                println!("❌ 区块链替换失败");
            }
        } else {
            println!("ℹ️ 新链不更长，保持当前链");
        }
    }

    /// 处理链长度响应
    fn handle_chain_length_response(blockchain: &Arc<Mutex<Blockchain>>, remote_length: usize, stream: &mut TcpStream, peer_addr: SocketAddr) {
        let blockchain = blockchain.lock().unwrap();
        let local_length = blockchain.get_length();

        println!("🔄 比较链长度: 本地={}, 远程={}", local_length, remote_length);

        if remote_length > local_length {
            println!("📈 远程链更长，需要同步");
            // 请求完整的区块链
            let request_message = Message::RequestChain;
            match serialize(&request_message) {
                Ok(data) => {
                    if let Err(e) = stream.write_all(&data) {
                        println!("❌ 请求区块链失败: {}", e);
                    } else {
                        println!("📤 请求完整区块链从 {}", peer_addr);
                    }
                }
                Err(e) => {
                    println!("❌ 序列化区块链请求失败: {}", e);
                }
            }
        } else if remote_length < local_length {
            println!("📈 本地链更长，考虑广播我们的链");
            // 广播我们的完整链
            let chain = blockchain.chain.clone();
            let response_message = Message::ResponseChain(chain);
            match serialize(&response_message) {
                Ok(data) => {
                    if let Err(e) = stream.write_all(&data) {
                        println!("❌ 广播区块链失败: {}", e);
                    } else {
                        println!("📤 广播完整区块链到 {}", peer_addr);
                    }
                }
                Err(e) => {
                    println!("❌ 序列化区块链响应失败: {}", e);
                }
            }
        } else {
            println!("📊 链长度相同，检查最新区块哈希");
            // 请求远程最新区块进行比较
            let request_message = Message::RequestBlocks { start: remote_length as u64 - 1, end: remote_length as u64 - 1 };
            match serialize(&request_message) {
                Ok(data) => {
                    if let Err(e) = stream.write_all(&data) {
                        println!("❌ 请求最新区块失败: {}", e);
                    } else {
                        println!("📤 请求最新区块从 {}", peer_addr);
                    }
                }
                Err(e) => {
                    println!("❌ 序列化区块请求失败: {}", e);
                }
            }
        }
    }

    /// 处理区块响应
    fn handle_blocks_response(blockchain: &mut Blockchain, blocks: Vec<Block>) {
        println!("🔄 处理区块响应，数量: {}", blocks.len());

        if blocks.is_empty() {
            println!("❌ 收到空区块列表");
            return;
        }

        // 验证区块序列
        for (i, block) in blocks.iter().enumerate() {
            if i == 0 {
                // 第一个区块应该连接到当前链
                let latest_block = blockchain.get_latest_block();
                if block.header.prev_hash != latest_block.header.hash {
                    println!("❌ 区块 {} 不连接到当前链", block.height);
                    return;
                }
            } else {
                // 后续区块应该连接到前一个区块
                let prev_block = &blocks[i - 1];
                if block.header.prev_hash != prev_block.header.hash {
                    println!("❌ 区块序列断裂在区块 {}", block.height);
                    return;
                }
            }
        }

        // 添加区块到链
        for block in &blocks {
            blockchain.chain.push(block.clone());
            println!("✅ 添加区块 {} 到链", block.height);
        }

        println!("✅ 成功添加 {} 个区块", blocks.len());
    }

    /// 处理同步状态
    fn handle_sync_status(
        blockchain: &Arc<Mutex<Blockchain>>,
        remote_length: usize,
        remote_hash: String,
        remote_transactions: usize,
    ) {
        let blockchain = blockchain.lock().unwrap();
        let local_length = blockchain.get_length();
        let local_transactions = blockchain.get_total_transactions();
        let local_hash = blockchain.get_latest_block().header.hash.clone();

        println!("🔄 同步状态比较:");
        println!("  本地: 长度={}, 哈希={}, 交易={}",
                 local_length, local_hash, local_transactions);
        println!("  远程: 长度={}, 哈希={}, 交易={}",
                 remote_length, remote_hash, remote_transactions);

        // 决定是否需要同步
        if remote_length > local_length ||
           (remote_length == local_length && remote_hash != local_hash) {
            println!("📈 需要同步到更新的链");
            // TODO: 触发同步逻辑
        } else {
            println!("✅ 本地链是最新的");
        }
    }

    /// 请求区块链同步
    pub fn request_chain_sync(&self, peer_address: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 请求与节点 {} 同步", peer_address);

        let message = Message::RequestChainLength;
        let data = serialize(&message)?;
        self.send_to_peer(peer_address, &data)?;

        Ok(())
    }

    /// 广播同步状态
    pub fn broadcast_sync_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let blockchain = self.blockchain.lock().unwrap();
        let chain_length = blockchain.get_length();
        let latest_hash = blockchain.get_latest_block().header.hash.clone();
        let total_transactions = blockchain.get_total_transactions();

        let message = Message::SyncStatus {
            chain_length,
            latest_hash: latest_hash.clone(),
            total_transactions,
        };

        self.broadcast_message(message)?;
        println!("📡 广播同步状态: 长度={}, 哈希={}, 交易={}",
                 chain_length, latest_hash, total_transactions);

        Ok(())
    }

    /// 启动同步流程
    pub fn start_sync_with_peer(&self, peer_address: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 开始与节点 {} 的同步流程", peer_address);

        // 1. 请求链长度
        let length_message = Message::RequestChainLength;
        let length_data = serialize(&length_message)?;
        self.send_to_peer(peer_address, &length_data)?;

        // 2. 广播我们的状态
        self.broadcast_sync_status()?;

        Ok(())
    }
}
