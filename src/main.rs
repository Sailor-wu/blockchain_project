mod block;
mod blockchain;
mod solana_program;
mod p2p_node;
mod cli;

use blockchain::Blockchain;
use p2p_node::P2PNode;
use cli::{add_transaction_cli, mine_block_cli, view_balance_cli, solana_demo, p2p_menu};
use std::sync::{Arc, Mutex};
use std::io::{self, Write};

/// 初始化区块链
fn initialize_blockchain() -> Blockchain {
    match Blockchain::load_from_file("blockchain.json") {
        Ok(loaded_blockchain) => {
            println!("✅ 从文件加载区块链成功!");
            loaded_blockchain
        }
        Err(_) => {
            println!("📁 区块链文件不存在，创建新区块链...");
            Blockchain::new(4, 100)
        }
    }
}

/// 初始化 P2P 节点
fn initialize_p2p_node(blockchain: &Arc<Mutex<Blockchain>>) -> P2PNode {
    P2PNode::new("127.0.0.1:7878".parse().unwrap(), blockchain.clone())
}

/// 主循环
fn run_main_loop(blockchain: &Arc<Mutex<Blockchain>>, p2p_node: &mut P2PNode) {
    loop {
        println!("\n请选择操作:");
        println!("1. 添加交易");
        println!("2. 挖矿");
        println!("3. 查看余额");
        println!("4. 查看区块链");
        println!("5. 验证区块链");
        println!("6. 保存区块链");
        println!("7. Solana 智能合约演示");
        println!("8. P2P 网络操作");
        println!("9. 退出");
        print!("输入选择 (1-9): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => add_transaction_cli(blockchain),
            "2" => mine_block_cli(blockchain),
            "3" => view_balance_cli(blockchain),
            "4" => {
                blockchain.lock().unwrap().print_chain();
            }
            "5" => {
                if blockchain.lock().unwrap().is_chain_valid() {
                    println!("✅ 区块链验证通过 - 所有区块都有效!");
                } else {
                    println!("❌ 区块链验证失败!");
                }
            }
            "6" => {
                match blockchain.lock().unwrap().save_to_file("blockchain.json") {
                    Ok(_) => println!("✅ 区块链保存成功!"),
                    Err(e) => println!("❌ 保存失败: {}", e),
                }
            }
            "7" => solana_demo(),
            "8" => p2p_menu(blockchain, p2p_node),
            "9" => {
                println!("👋 再见!");
                break;
            }
            _ => println!("❌ 无效选择，请重新输入."),
        }
    }
}

fn main() {
    println!("🚀 欢迎使用 Rust 区块链 CLI!");
    println!("=====================================\n");

    // 初始化区块链
    let blockchain = initialize_blockchain();
    let blockchain_arc = Arc::new(Mutex::new(blockchain));

    // 初始化 P2P 节点
    let mut p2p_node = initialize_p2p_node(&blockchain_arc);

    // 启动主循环
    run_main_loop(&blockchain_arc, &mut p2p_node);
}
