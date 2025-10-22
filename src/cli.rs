use crate::blockchain::Blockchain;
use crate::block::Transaction;
use crate::p2p_node::P2PNode;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::io::{self, Write};

/// 添加交易 CLI
pub fn add_transaction_cli(blockchain: &Arc<Mutex<Blockchain>>) {
    print!("输入发送者地址: ");
    io::stdout().flush().unwrap();
    let mut sender = String::new();
    io::stdin().read_line(&mut sender).unwrap();
    let sender = sender.trim().to_string();

    print!("输入接收者地址: ");
    io::stdout().flush().unwrap();
    let mut receiver = String::new();
    io::stdin().read_line(&mut receiver).unwrap();
    let receiver = receiver.trim().to_string();

    print!("输入交易金额: ");
    io::stdout().flush().unwrap();
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str).unwrap();
    let amount: u64 = match amount_str.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("❌ 无效金额");
            return;
        }
    };

    let transaction = Transaction::new(sender, receiver, amount);
    match blockchain.lock().unwrap().add_transaction(transaction) {
        Ok(_) => println!("✅ 交易添加成功!"),
        Err(e) => println!("❌ 交易添加失败: {}", e),
    }
}

/// 挖矿 CLI
pub fn mine_block_cli(blockchain: &Arc<Mutex<Blockchain>>) {
    print!("输入矿工地址: ");
    io::stdout().flush().unwrap();
    let mut miner = String::new();
    io::stdin().read_line(&mut miner).unwrap();
    let miner = miner.trim().to_string();

    match blockchain.lock().unwrap().mine_pending_transactions(miner) {
        Ok(block) => {
            println!("✅ 新区块挖矿成功!");
            println!("区块信息: {}", block);
        }
        Err(e) => println!("❌ 挖矿失败: {}", e),
    }
}

/// 查看余额 CLI
pub fn view_balance_cli(blockchain: &Arc<Mutex<Blockchain>>) {
    print!("输入地址: ");
    io::stdout().flush().unwrap();
    let mut address = String::new();
    io::stdin().read_line(&mut address).unwrap();
    let address = address.trim();

    let balance = blockchain.lock().unwrap().get_balance(address);
    println!("{} 的余额: {}", address, balance);
}

/// Solana 演示 CLI
pub fn solana_demo() {
    println!("\n🔗 Solana 智能合约演示");
    println!("=====================================");
    println!("这是一个简单的 Solana 智能合约示例，实现了基本的转账功能。");
    println!("\n📋 合约特性:");
    println!("- 使用 Rust 编写");
    println!("- 实现账户间转账");
    println!("- 包含余额检查");
    println!("- 错误处理和日志记录");
    println!("\n💡 部署说明:");
    println!("1. 安装 Solana CLI: solana --version");
    println!("2. 启动本地网络: solana-test-validator");
    println!("3. 构建合约: cargo build-bpf");
    println!("4. 部署合约: solana program deploy target/deploy/*.so");
    println!("\n📖 学习资源:");
    println!("- Solana 官方文档: https://docs.solana.com/");
    println!("- Rust 智能合约教程: https://solana.com/developers");
    println!("\n按回车键返回主菜单...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

/// P2P 菜单 CLI
pub fn p2p_menu(_blockchain: &Arc<Mutex<Blockchain>>, p2p_node: &mut P2PNode) {
    loop {
        println!("\n🌐 P2P 网络操作");
        println!("=====================================");
        println!("1. 启动 P2P 节点");
        println!("2. 连接到其他节点");
        println!("3. 查看对等节点");
        println!("4. 广播交易");
        println!("5. 广播区块");
        println!("6. 返回主菜单");
        print!("输入选择 (1-6): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => {
                if let Err(e) = p2p_node.start() {
                    println!("❌ 启动 P2P 节点失败: {}", e);
                } else {
                    println!("✅ P2P 节点已启动");
                }
            }
            "2" => {
                print!("输入节点地址 (例如: 127.0.0.1:7879): ");
                io::stdout().flush().unwrap();
                let mut peer_input = String::new();
                io::stdin().read_line(&mut peer_input).unwrap();
                let peer_addr: SocketAddr = match peer_input.trim().parse() {
                    Ok(addr) => addr,
                    Err(_) => {
                        println!("❌ 无效地址格式");
                        continue;
                    }
                };

                if let Err(e) = p2p_node.connect_to_peer(peer_addr) {
                    println!("❌ 连接失败: {}", e);
                }
            }
            "3" => {
                let peers = p2p_node.get_peers();
                if peers.is_empty() {
                    println!("📭 没有连接的对等节点");
                } else {
                    println!("🔗 连接的对等节点:");
                    for peer in peers {
                        println!("  - {}", peer);
                    }
                }
            }
            "4" => {
                print!("输入发送者地址: ");
                io::stdout().flush().unwrap();
                let mut sender = String::new();
                io::stdin().read_line(&mut sender).unwrap();
                let sender = sender.trim().to_string();

                print!("输入接收者地址: ");
                io::stdout().flush().unwrap();
                let mut receiver = String::new();
                io::stdin().read_line(&mut receiver).unwrap();
                let receiver = receiver.trim().to_string();

                print!("输入交易金额: ");
                io::stdout().flush().unwrap();
                let mut amount_str = String::new();
                io::stdin().read_line(&mut amount_str).unwrap();
                let amount: u64 = match amount_str.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("❌ 无效金额");
                        continue;
                    }
                };

                let transaction = Transaction::new(sender, receiver, amount);
                if let Err(e) = p2p_node.broadcast_transaction(transaction) {
                    println!("❌ 广播交易失败: {}", e);
                } else {
                    println!("✅ 交易已广播到网络");
                }
            }
            "5" => {
                println!("💡 区块广播功能需要进一步实现");
            }
            "6" => break,
            _ => println!("❌ 无效选择，请重新输入."),
        }
    }
}
