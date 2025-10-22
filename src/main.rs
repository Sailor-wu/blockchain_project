mod block;
mod blockchain;

use blockchain::Blockchain;
use block::Transaction;

fn main() {
    println!("🚀 欢迎使用 Rust 区块链演示!");
    println!("=====================================\n");

    // 创建区块链实例
    let mut blockchain = Blockchain::new(4, 100);

    println!("1. 创建区块链...");
    blockchain.print_chain();

    // 创建一些交易
    println!("\n2. 创建交易...");

    // 为演示目的，给一些账户初始资金
    blockchain.balances.insert("Alice".to_string(), 1000);
    blockchain.balances.insert("Bob".to_string(), 500);

    // 添加交易
    let tx1 = Transaction::new("Alice".to_string(), "Bob".to_string(), 100);
    let tx2 = Transaction::new("Bob".to_string(), "Alice".to_string(), 50);

    match blockchain.add_transaction(tx1) {
        Ok(_) => println!("✅ 交易1添加成功"),
        Err(e) => println!("❌ 交易1失败: {}", e),
    }

    match blockchain.add_transaction(tx2) {
        Ok(_) => println!("✅ 交易2添加成功"),
        Err(e) => println!("❌ 交易2失败: {}", e),
    }

    // 挖矿
    println!("\n3. 开始挖矿...");
    match blockchain.mine_pending_transactions("Miner1".to_string()) {
        Ok(block) => {
            println!("✅ 新区块挖矿成功!");
            println!("区块信息: {}", block);
        }
        Err(e) => println!("❌ 挖矿失败: {}", e),
    }

    // 显示区块链状态
    blockchain.print_chain();

    // 创建更多交易和区块
    println!("\n4. 创建更多交易和区块...");

    let tx3 = Transaction::new("Alice".to_string(), "Bob".to_string(), 75);
    blockchain.add_transaction(tx3).unwrap();

    let tx4 = Transaction::new("Bob".to_string(), "Alice".to_string(), 25);
    blockchain.add_transaction(tx4).unwrap();

    // 挖矿第二个区块
    blockchain.mine_pending_transactions("Miner2".to_string()).unwrap();

    // 显示难度调整
    println!("\n4.5. 难度调整演示:");
    println!("当前难度: {}", blockchain.difficulty);

    // 模拟快速挖矿更多区块来触发难度调整
    for i in 3..=5 {
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 10 * i as u64);
        blockchain.add_transaction(tx).unwrap();
        blockchain.mine_pending_transactions(format!("Miner{}", i)).unwrap();
        println!("挖矿区块 {} 后，难度: {}", i, blockchain.difficulty);
    }

    // 最终状态
    println!("\n5. 最终区块链状态:");
    blockchain.print_chain();

    // 验证区块链
    println!("\n6. 验证区块链完整性...");
    if blockchain.is_chain_valid() {
        println!("✅ 区块链验证通过 - 所有区块都有效!");
    } else {
        println!("❌ 区块链验证失败!");
    }

    println!("\n🎉 区块链演示完成!");
    println!("=====================================");
}
