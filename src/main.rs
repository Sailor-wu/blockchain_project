mod block;
mod blockchain;
mod solana_program;
mod p2p_node;
mod cli;
mod consensus;

use blockchain::Blockchain;
use p2p_node::P2PNode;
use cli::{add_transaction_cli, mine_block_cli, view_balance_cli, solana_demo, p2p_menu};
use consensus::{ConsensusType, ProofOfStake, DelegatedProofOfStake};
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
        println!("9. 共识算法管理");
        println!("10. 退出");
        print!("输入选择 (1-10): ");
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
            "9" => consensus_menu(blockchain),
            "10" => {
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

/// 共识算法管理菜单
fn consensus_menu(blockchain: &Arc<Mutex<Blockchain>>) {
    loop {
        println!("\n⚖️ 共识算法管理");
        println!("=====================================");
        println!("当前共识算法: {:?}", blockchain.lock().unwrap().consensus_type);
        println!("\n1. 切换到 PoW (工作量证明)");
        println!("2. 切换到 PoS (权益证明)");
        println!("3. 切换到 DPoS (委托权益证明)");
        println!("4. PoS 质押管理");
        println!("5. DPoS 候选人管理");
        println!("6. 返回主菜单");
        print!("输入选择 (1-6): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => {
                blockchain.lock().unwrap().consensus_type = ConsensusType::PoW;
                blockchain.lock().unwrap().pos_consensus = None;
                blockchain.lock().unwrap().dpos_consensus = None;
                println!("✅ 已切换到 PoW 共识算法");
            }
            "2" => {
                let mut blockchain = blockchain.lock().unwrap();
                blockchain.consensus_type = ConsensusType::PoS;
                blockchain.pos_consensus = Some(ProofOfStake::new(100)); // 最小质押100
                blockchain.dpos_consensus = None;
                println!("✅ 已切换到 PoS 共识算法");
            }
            "3" => {
                let mut blockchain = blockchain.lock().unwrap();
                blockchain.consensus_type = ConsensusType::DPoS;
                blockchain.pos_consensus = None;
                blockchain.dpos_consensus = Some(DelegatedProofOfStake::new(1000, 100)); // 最小质押1000，委托100
                println!("✅ 已切换到 DPoS 共识算法");
            }
            "4" => pos_stake_menu(blockchain),
            "5" => dpos_candidate_menu(blockchain),
            "6" => break,
            _ => println!("❌ 无效选择，请重新输入."),
        }
    }
}

/// PoS 质押管理菜单
fn pos_stake_menu(blockchain: &Arc<Mutex<Blockchain>>) {
    loop {
        println!("\n💰 PoS 质押管理");
        println!("=====================================");
        println!("1. 质押代币");
        println!("2. 取消质押");
        println!("3. 查看质押信息");
        println!("4. 返回上级菜单");
        print!("输入选择 (1-4): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => {
                print!("输入验证者地址: ");
                io::stdout().flush().unwrap();
                let mut validator = String::new();
                io::stdin().read_line(&mut validator).unwrap();
                let validator = validator.trim().to_string();

                print!("输入质押金额: ");
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

                if let Some(ref mut pos) = blockchain.lock().unwrap().pos_consensus {
                    match pos.stake(validator, amount) {
                        Ok(_) => println!("✅ 质押成功!"),
                        Err(e) => println!("❌ 质押失败: {}", e),
                    }
                }
            }
            "2" => {
                print!("输入验证者地址: ");
                io::stdout().flush().unwrap();
                let mut validator = String::new();
                io::stdin().read_line(&mut validator).unwrap();
                let validator = validator.trim().to_string();

                if let Some(ref mut pos) = blockchain.lock().unwrap().pos_consensus {
                    match pos.unstake(validator) {
                        Ok(_) => println!("✅ 取消质押成功!"),
                        Err(e) => println!("❌ 取消质押失败: {}", e),
                    }
                }
            }
            "3" => {
                if let Some(ref pos) = blockchain.lock().unwrap().pos_consensus {
                    println!("📋 质押信息:");
                    for (validator, stake_info) in &pos.stakes {
                        println!("  验证者: {} - 金额: {} - 时间: {}",
                                validator, stake_info.amount, stake_info.start_time);
                    }
                } else {
                    println!("❌ 当前未使用 PoS 共识算法");
                }
            }
            "4" => break,
            _ => println!("❌ 无效选择，请重新输入."),
        }
    }
}

/// DPoS 候选人管理菜单
fn dpos_candidate_menu(blockchain: &Arc<Mutex<Blockchain>>) {
    loop {
        println!("\n🏛️ DPoS 候选人管理");
        println!("=====================================");
        println!("1. 注册候选人");
        println!("2. 委托投票");
        println!("3. 查看候选人");
        println!("4. 查看委托信息");
        println!("5. 返回上级菜单");
        print!("输入选择 (1-5): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => {
                print!("输入候选人地址: ");
                io::stdout().flush().unwrap();
                let mut candidate = String::new();
                io::stdin().read_line(&mut candidate).unwrap();
                let candidate = candidate.trim().to_string();

                print!("输入质押金额: ");
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

                if let Some(ref mut dpos) = blockchain.lock().unwrap().dpos_consensus {
                    match dpos.register_candidate(candidate, amount) {
                        Ok(_) => println!("✅ 候选人注册成功!"),
                        Err(e) => println!("❌ 候选人注册失败: {}", e),
                    }
                }
            }
            "2" => {
                print!("输入委托人地址: ");
                io::stdout().flush().unwrap();
                let mut delegator = String::new();
                io::stdin().read_line(&mut delegator).unwrap();
                let delegator = delegator.trim().to_string();

                print!("输入候选人地址: ");
                io::stdout().flush().unwrap();
                let mut candidate = String::new();
                io::stdin().read_line(&mut candidate).unwrap();
                let candidate = candidate.trim().to_string();

                print!("输入委托金额: ");
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

                if let Some(ref mut dpos) = blockchain.lock().unwrap().dpos_consensus {
                    match dpos.delegate(delegator, candidate, amount) {
                        Ok(_) => println!("✅ 委托投票成功!"),
                        Err(e) => println!("❌ 委托投票失败: {}", e),
                    }
                }
            }
            "3" => {
                if let Some(ref dpos) = blockchain.lock().unwrap().dpos_consensus {
                    println!("📋 候选人列表:");
                    for candidate in &dpos.candidates {
                        let weight = dpos.calculate_candidate_weight(candidate);
                        println!("  候选人: {} - 权重: {}", candidate, weight);
                    }
                } else {
                    println!("❌ 当前未使用 DPoS 共识算法");
                }
            }
            "4" => {
                if let Some(ref dpos) = blockchain.lock().unwrap().dpos_consensus {
                    println!("📋 委托信息:");
                    for (key, delegation) in &dpos.delegations {
                        println!("  {} -> {}: {}", delegation.delegator, delegation.candidate, delegation.amount);
                    }
                } else {
                    println!("❌ 当前未使用 DPoS 共识算法");
                }
            }
            "5" => break,
            _ => println!("❌ 无效选择，请重新输入."),
        }
    }
}
