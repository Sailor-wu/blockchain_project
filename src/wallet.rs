use crate::block::Transaction;
use ring::signature::{Ed25519KeyPair, KeyPair};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use hex;

/// 钱包结构体 - 管理用户的密钥对和地址
#[derive(Debug, Clone)]
pub struct Wallet {
    pub address: String,
    pub public_key: String,
    pub encrypted_private_key: String, // 在实际项目中应该加密存储
}

impl Wallet {
    /// 创建新钱包
    pub fn new(name: String) -> Self {
        let keypair = Transaction::generate_keypair();
        let public_key = hex::encode(keypair.public_key().as_ref());

        Self {
            address: name.clone(),
            public_key: public_key.clone(),
            encrypted_private_key: hex::encode(keypair.public_key().as_ref()), // 简化版，实际应该加密私钥
        }
    }

    /// 从私钥恢复钱包（简化版）
    pub fn from_private_key(name: String, private_key_hex: &str) -> Result<Self, String> {
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|_| "无效的私钥格式".to_string())?;

        // 简化版：实际应该使用私钥重新生成密钥对
        let keypair = Transaction::generate_keypair();
        let public_key = hex::encode(keypair.public_key().as_ref());

        Ok(Self {
            address: name,
            public_key,
            encrypted_private_key: private_key_hex.to_string(),
        })
    }

    /// 获取钱包地址
    pub fn get_address(&self) -> &str {
        &self.address
    }

    /// 获取公钥
    pub fn get_public_key(&self) -> &str {
        &self.public_key
    }
}

/// 钱包管理器 - 管理多个钱包
pub struct WalletManager {
    wallets: Arc<Mutex<HashMap<String, Wallet>>>,
}

impl WalletManager {
    /// 创建新的钱包管理器
    pub fn new() -> Self {
        Self {
            wallets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建新钱包
    pub fn create_wallet(&self, name: String) -> Result<String, String> {
        let mut wallets = self.wallets.lock().unwrap();

        if wallets.contains_key(&name) {
            return Err(format!("钱包 '{}' 已存在", name));
        }

        let wallet = Wallet::new(name.clone());
        let public_key = wallet.public_key.clone();

        wallets.insert(name.clone(), wallet);

        println!("✅ 钱包 '{}' 创建成功!", name);
        println!("📬 钱包地址: {}", name);
        println!("🔑 公钥: {}", public_key);

        Ok(public_key)
    }

    /// 导入钱包
    pub fn import_wallet(&self, name: String, private_key_hex: String) -> Result<String, String> {
        let mut wallets = self.wallets.lock().unwrap();

        if wallets.contains_key(&name) {
            return Err(format!("钱包 '{}' 已存在", name));
        }

        let wallet = Wallet::from_private_key(name.clone(), &private_key_hex)?;
        let public_key = wallet.public_key.clone();

        wallets.insert(name.clone(), wallet);

        println!("✅ 钱包 '{}' 导入成功!", name);
        println!("📬 钱包地址: {}", name);
        println!("🔑 公钥: {}", public_key);

        Ok(public_key)
    }

    /// 获取钱包
    pub fn get_wallet(&self, name: &str) -> Option<Wallet> {
        let wallets = self.wallets.lock().unwrap();
        wallets.get(name).cloned()
    }

    /// 列出所有钱包
    pub fn list_wallets(&self) -> Vec<String> {
        let wallets = self.wallets.lock().unwrap();
        wallets.keys().cloned().collect()
    }

    /// 删除钱包
    pub fn delete_wallet(&self, name: &str) -> Result<(), String> {
        let mut wallets = self.wallets.lock().unwrap();

        if !wallets.contains_key(name) {
            return Err(format!("钱包 '{}' 不存在", name));
        }

        wallets.remove(name);
        println!("✅ 钱包 '{}' 已删除", name);
        Ok(())
    }

    /// 获取钱包数量
    pub fn wallet_count(&self) -> usize {
        let wallets = self.wallets.lock().unwrap();
        wallets.len()
    }
}

/// 钱包 CLI 功能

/// 创建钱包 CLI
pub fn create_wallet_cli(wallet_manager: &WalletManager) {
    println!("\n👛 创建钱包");
    println!("=====================================");

    print!("输入钱包名称: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim().to_string();

    if name.is_empty() {
        println!("❌ 钱包名称不能为空");
        return;
    }

    match wallet_manager.create_wallet(name) {
        Ok(_) => {
            println!("💡 请妥善保管钱包信息!");
            println!("   在实际项目中，私钥应该加密存储");
        }
        Err(e) => println!("❌ 创建钱包失败: {}", e),
    }
}

/// 导入钱包 CLI
pub fn import_wallet_cli(wallet_manager: &WalletManager) {
    println!("\n📥 导入钱包");
    println!("=====================================");

    print!("输入钱包名称: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim().to_string();

    if name.is_empty() {
        println!("❌ 钱包名称不能为空");
        return;
    }

    print!("输入私钥 (十六进制): ");
    io::stdout().flush().unwrap();
    let mut private_key = String::new();
    io::stdin().read_line(&mut private_key).unwrap();
    let private_key = private_key.trim().to_string();

    if private_key.is_empty() {
        println!("❌ 私钥不能为空");
        return;
    }

    match wallet_manager.import_wallet(name, private_key) {
        Ok(_) => println!("💡 钱包导入成功!"),
        Err(e) => println!("❌ 导入钱包失败: {}", e),
    }
}

/// 查看钱包 CLI
pub fn view_wallet_cli(wallet_manager: &WalletManager) {
    println!("\n👛 查看钱包");
    println!("=====================================");

    print!("输入钱包名称: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim().to_string();

    match wallet_manager.get_wallet(&name) {
        Some(wallet) => {
            println!("✅ 钱包信息:");
            println!("📬 钱包地址: {}", wallet.address);
            println!("🔑 公钥: {}", wallet.public_key);
            println!("🔒 私钥哈希: {}", hex::encode(&wallet.encrypted_private_key[..8])); // 只显示前8字节
        }
        None => println!("❌ 未找到钱包 '{}'", name),
    }
}

/// 列出钱包 CLI
pub fn list_wallets_cli(wallet_manager: &WalletManager) {
    println!("\n📋 钱包列表");
    println!("=====================================");

    let wallets = wallet_manager.list_wallets();

    if wallets.is_empty() {
        println!("📭 暂无钱包，请先创建钱包");
        return;
    }

    println!("找到 {} 个钱包:", wallets.len());
    for (i, wallet_name) in wallets.iter().enumerate() {
        if let Some(wallet) = wallet_manager.get_wallet(wallet_name) {
            println!("{}. 📬 {} - 🔑 {}", i + 1, wallet.address, &wallet.public_key[..16]);
        }
    }
}

/// 删除钱包 CLI
pub fn delete_wallet_cli(wallet_manager: &WalletManager) {
    println!("\n🗑️ 删除钱包");
    println!("=====================================");
    println!("⚠️ 警告：删除钱包将丢失所有关联的资产!");

    print!("输入要删除的钱包名称: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim().to_string();

    print!("确认删除钱包 '{}' ? (输入 'yes' 确认): ", name);
    io::stdout().flush().unwrap();
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();
    let confirm = confirm.trim().to_string();

    if confirm == "yes" {
        match wallet_manager.delete_wallet(&name) {
            Ok(_) => println!("💡 钱包删除成功"),
            Err(e) => println!("❌ 删除钱包失败: {}", e),
        }
    } else {
        println!("❌ 删除操作已取消");
    }
}

/// 钱包管理菜单
pub fn wallet_menu(wallet_manager: &WalletManager) {
    loop {
        println!("\n👛 钱包管理");
        println!("=====================================");
        println!("钱包数量: {}", wallet_manager.wallet_count());
        println!("\n1. 创建新钱包");
        println!("2. 导入钱包");
        println!("3. 查看钱包");
        println!("4. 列出所有钱包");
        println!("5. 删除钱包");
        println!("6. 返回主菜单");
        print!("输入选择 (1-6): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => create_wallet_cli(wallet_manager),
            "2" => import_wallet_cli(wallet_manager),
            "3" => view_wallet_cli(wallet_manager),
            "4" => list_wallets_cli(wallet_manager),
            "5" => delete_wallet_cli(wallet_manager),
            "6" => break,
            _ => println!("❌ 无效选择，请重新输入."),
        }
    }
}
