mod block;
mod blockchain;
mod solana_program;

use blockchain::Blockchain;
use block::Transaction;

use std::io::{self, Write};

fn main() {
    println!("🚀 欢迎使用 Rust 区块链 CLI!");
    println!("=====================================\n");

    // 尝试从文件加载区块链，如果失败则创建新区块链
    let mut blockchain = match Blockchain::load_from_file("blockchain.json") {
        Ok(loaded_blockchain) => {
            println!("✅ 从文件加载区块链成功!");
            loaded_blockchain
        }
        Err(_) => {
            println!("📁 区块链文件不存在，创建新区块链...");
            Blockchain::new(4, 100)
        }
    };

    loop {
        println!("\n请选择操作:");
        println!("1. 添加交易");
        println!("2. 挖矿");
        println!("3. 查看余额");
        println!("4. 查看区块链");
        println!("5. 验证区块链");
        println!("6. 保存区块链");
        println!("7. Solana 智能合约演示");
        println!("8. 退出");
        print!("输入选择 (1-8): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => add_transaction_cli(&mut blockchain),
            "2" => mine_block_cli(&mut blockchain),
            "3" => view_balance_cli(&blockchain),
            "4" => {
                blockchain.print_chain();
            }
            "5" => {
                if blockchain.is_chain_valid() {
                    println!("✅ 区块链验证通过 - 所有区块都有效!");
                } else {
                    println!("❌ 区块链验证失败!");
                }
            }
            "6" => {
                match blockchain.save_to_file("blockchain.json") {
                    Ok(_) => println!("✅ 区块链保存成功!"),
                    Err(e) => println!("❌ 保存失败: {}", e),
                }
            }
            "7" => solana_demo(),
            "8" => {
                println!("👋 再见!");
                break;
            }
            _ => println!("❌ 无效选择，请重新输入."),
        }
    }
}

fn add_transaction_cli(blockchain: &mut Blockchain) {
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
    match blockchain.add_transaction(transaction) {
        Ok(_) => println!("✅ 交易添加成功!"),
        Err(e) => println!("❌ 交易添加失败: {}", e),
    }
}

fn mine_block_cli(blockchain: &mut Blockchain) {
    print!("输入矿工地址: ");
    io::stdout().flush().unwrap();
    let mut miner = String::new();
    io::stdin().read_line(&mut miner).unwrap();
    let miner = miner.trim().to_string();

    match blockchain.mine_pending_transactions(miner) {
        Ok(block) => {
            println!("✅ 新区块挖矿成功!");
            println!("区块信息: {}", block);
        }
        Err(e) => println!("❌ 挖矿失败: {}", e),
    }
}

fn view_balance_cli(blockchain: &Blockchain) {
    print!("输入地址: ");
    io::stdout().flush().unwrap();
    let mut address = String::new();
    io::stdin().read_line(&mut address).unwrap();
    let address = address.trim();

    let balance = blockchain.get_balance(address);
    println!("{} 的余额: {}", address, balance);
}

fn solana_demo() {
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
