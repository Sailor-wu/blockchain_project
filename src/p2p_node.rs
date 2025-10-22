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
    /// 节点发现
    Discovery(SocketAddr),
    /// 心跳消息
    Ping,
    /// 心跳响应
    Pong,
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
                Ok(mut stream) => {
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
        let mut buffer = [0; 1024];

        loop {
            match stream.read(&mut buffer) {
                Ok(size) if size > 0 => {
                    let data = &buffer[..size];
                    match deserialize::<Message>(data) {
                        Ok(message) => {
                            if let Err(e) = Self::process_message(message, &blockchain, &peers) {
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
                // 这里可以添加区块验证和替换逻辑
                if blockchain.is_chain_valid() {
                    println!("✅ 区块验证通过");
                }
            }
            Message::RequestChain => {
                println!("📋 收到区块链请求");
                let blockchain = blockchain.lock().unwrap();
                let _chain = blockchain.chain.clone();
                // 发送区块链响应
                // 这里应该发送 ResponseChain 消息
            }
            Message::Discovery(peer_addr) => {
                println!("🔍 发现新节点: {}", peer_addr);
                peers.lock().unwrap().insert(peer_addr, PeerInfo {
                    address: peer_addr,
                    last_seen: std::time::Instant::now(),
                });
            }
            Message::Ping => {
                // 响应 Pong
                println!("🏓 收到 Ping");
            }
            Message::Pong => {
                println!("🏓 收到 Pong");
            }
            _ => {
                println!("❓ 未知消息类型");
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
}
