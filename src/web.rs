use crate::blockchain::Blockchain;
use crate::block::{Block, Transaction};
use crate::wallet::WalletManager;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

/// Web服务器状态
#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub wallet_manager: Arc<WalletManager>,
}

/// API响应结构体
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
struct BlockchainInfo {
    length: usize,
    total_transactions: usize,
    difficulty: u32,
    mining_reward: u64,
    is_valid: bool,
    consensus_type: String,
}

#[derive(Serialize)]
struct BlockInfo {
    height: u64,
    hash: String,
    prev_hash: String,
    timestamp: String,
    nonce: u64,
    difficulty: u32,
    transaction_count: usize,
    transactions: Vec<TransactionInfo>,
}

#[derive(Serialize)]
struct TransactionInfo {
    id: String,
    sender: String,
    receiver: String,
    amount: u64,
    timestamp: String,
    has_signature: bool,
}

#[derive(Serialize)]
struct BalanceInfo {
    address: String,
    balance: u64,
}

#[derive(Deserialize)]
struct CreateTransactionRequest {
    sender: String,
    receiver: String,
    amount: u64,
}

#[derive(Deserialize)]
struct MineRequest {
    miner_address: String,
}

/// 获取区块链信息
async fn get_blockchain_info(
    State(state): State<AppState>,
) -> Json<ApiResponse<BlockchainInfo>> {
    let blockchain = state.blockchain.lock().unwrap();

    let info = BlockchainInfo {
        length: blockchain.get_length(),
        total_transactions: blockchain.get_total_transactions(),
        difficulty: blockchain.difficulty,
        mining_reward: blockchain.mining_reward,
        is_valid: blockchain.is_chain_valid(),
        consensus_type: format!("{:?}", blockchain.consensus_type),
    };

    Json(ApiResponse {
        success: true,
        data: Some(info),
        error: None,
    })
}

/// 获取所有区块
async fn get_blocks(State(state): State<AppState>) -> Json<ApiResponse<Vec<BlockInfo>>> {
    let blockchain = state.blockchain.lock().unwrap();
    let mut blocks = Vec::new();

    for block in &blockchain.chain {
        let block_info = BlockInfo {
            height: block.height,
            hash: block.header.hash.clone(),
            prev_hash: block.header.prev_hash.clone(),
            timestamp: block.header.timestamp.to_rfc3339(),
            nonce: block.header.nonce,
            difficulty: block.header.difficulty,
            transaction_count: block.transactions.len(),
            transactions: block.transactions.iter().map(|tx| TransactionInfo {
                id: tx.id.clone(),
                sender: tx.sender.clone(),
                receiver: tx.receiver.clone(),
                amount: tx.amount,
                timestamp: tx.timestamp.to_rfc3339(),
                has_signature: tx.signature.is_some(),
            }).collect(),
        };
        blocks.push(block_info);
    }

    Json(ApiResponse {
        success: true,
        data: Some(blocks),
        error: None,
    })
}

/// 获取特定区块
async fn get_block(
    State(state): State<AppState>,
    Path(height): Path<usize>,
) -> Json<ApiResponse<BlockInfo>> {
    let blockchain = state.blockchain.lock().unwrap();

    if height >= blockchain.chain.len() {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("区块不存在".to_string()),
        });
    }

    let block = &blockchain.chain[height];
    let block_info = BlockInfo {
        height: block.height,
        hash: block.header.hash.clone(),
        prev_hash: block.header.prev_hash.clone(),
        timestamp: block.header.timestamp.to_rfc3339(),
        nonce: block.header.nonce,
        difficulty: block.header.difficulty,
        transaction_count: block.transactions.len(),
        transactions: block.transactions.iter().map(|tx| TransactionInfo {
            id: tx.id.clone(),
            sender: tx.sender.clone(),
            receiver: tx.receiver.clone(),
            amount: tx.amount,
            timestamp: tx.timestamp.to_rfc3339(),
            has_signature: tx.signature.is_some(),
        }).collect(),
    };

    Json(ApiResponse {
        success: true,
        data: Some(block_info),
        error: None,
    })
}

/// 获取账户余额
async fn get_balance(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Json<ApiResponse<BalanceInfo>> {
    let blockchain = state.blockchain.lock().unwrap();
    let balance = blockchain.get_balance(&address);

    let balance_info = BalanceInfo {
        address,
        balance,
    };

    Json(ApiResponse {
        success: true,
        data: Some(balance_info),
        error: None,
    })
}

/// 获取待处理交易
async fn get_pending_transactions(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<TransactionInfo>>> {
    let blockchain = state.blockchain.lock().unwrap();

    let transactions = blockchain.pending_transactions.iter().map(|tx| TransactionInfo {
        id: tx.id.clone(),
        sender: tx.sender.clone(),
        receiver: tx.receiver.clone(),
        amount: tx.amount,
        timestamp: tx.timestamp.to_rfc3339(),
        has_signature: tx.signature.is_some(),
    }).collect();

    Json(ApiResponse {
        success: true,
        data: Some(transactions),
        error: None,
    })
}

/// 创建新交易
async fn create_transaction(
    State(state): State<AppState>,
    Json(request): Json<CreateTransactionRequest>,
) -> Json<ApiResponse<String>> {
    let mut blockchain = state.blockchain.lock().unwrap();

    let transaction = Transaction::new(
        request.sender,
        request.receiver,
        request.amount,
    );

    match blockchain.add_transaction(transaction) {
        Ok(_) => Json(ApiResponse {
            success: true,
            data: Some("交易创建成功".to_string()),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

/// 执行挖矿
async fn mine_block(
    State(state): State<AppState>,
    Json(request): Json<MineRequest>,
) -> Json<ApiResponse<String>> {
    let mut blockchain = state.blockchain.lock().unwrap();

    match blockchain.mine_pending_transactions(request.miner_address) {
        Ok(_) => Json(ApiResponse {
            success: true,
            data: Some("挖矿成功".to_string()),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

async fn index() -> Html<&'static str> {
    Html("<!DOCTYPE html>
<html><head><title>Rust区块链系统</title></head>
<body>
<h1>🚀 Rust 区块链系统</h1>
<p>欢迎使用功能强大的区块链技术演示平台！</p>
<a href='/explorer'>🌐 进入区块链浏览器</a>
</body></html>")
}

async fn blockchain_explorer() -> Html<&'static str> {
    Html("<!DOCTYPE html>
<html><head><title>区块链浏览器</title></head>
<body>
<h1>🔗 区块链浏览器</h1>
<p>实时区块链数据可视化平台</p>
<h2>📊 区块链统计</h2>
<div id='stats'>加载中...</div>
<h2>⛏️ 挖矿操作</h2>
<input type='text' id='minerAddress' placeholder='矿工地址'>
<button onclick='mineBlock()'>开始挖矿</button>
<div id='miningResult'></div>
<h2>💸 创建交易</h2>
<input type='text' id='sender' placeholder='发送者'>
<input type='text' id='receiver' placeholder='接收者'>
<input type='number' id='amount' placeholder='金额'>
<button onclick='createTransaction()'>创建交易</button>
<div id='transactionResult'></div>
<h2>📦 区块列表</h2>
<div id='blocks'>加载中...</div>
<script>
async function loadStats() {
    const response = await fetch('/api/blockchain/info');
    const data = await response.json();
    document.getElementById('stats').innerHTML =
        `<p>区块高度: ${data.data.length}</p>
         <p>总交易数: ${data.data.total_transactions}</p>
         <p>挖矿难度: ${data.data.difficulty}</p>`;
}

async function mineBlock() {
    const addr = document.getElementById('minerAddress').value;
    const response = await fetch('/api/mine', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({miner_address: addr})
    });
    const data = await response.json();
    document.getElementById('miningResult').innerHTML =
        data.success ? '✅ 挖矿成功' : '❌ 挖矿失败: ' + data.error;
    loadStats();
}

async function createTransaction() {
    const tx = {
        sender: document.getElementById('sender').value,
        receiver: document.getElementById('receiver').value,
        amount: parseInt(document.getElementById('amount').value)
    };
    const response = await fetch('/api/transactions', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify(tx)
    });
    const data = await response.json();
    document.getElementById('transactionResult').innerHTML =
        data.success ? '✅ 交易创建成功' : '❌ 创建失败: ' + data.error;
}

loadStats();
setInterval(loadStats, 5000);
</script>
</body></html>")
}

/// 启动Web服务器
pub async fn start_web_server(
    blockchain: Arc<Mutex<Blockchain>>,
    wallet_manager: Arc<WalletManager>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        blockchain,
        wallet_manager,
    };

    // 创建路由
    let app = Router::new()
        .route("/", get(index))
        .route("/explorer", get(blockchain_explorer))
        .route("/api/blockchain/info", get(get_blockchain_info))
        .route("/api/blocks", get(get_blocks))
        .route("/api/blocks/:height", get(get_block))
        .route("/api/balance/:address", get(get_balance))
        .route("/api/pending-transactions", get(get_pending_transactions))
        .route("/api/transactions", post(create_transaction))
        .route("/api/mine", post(mine_block))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // 添加静态文件服务
    let app = app.nest_service("/static", ServeDir::new("static"));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🌐 区块链浏览器启动中...");
    println!("📡 服务器地址: http://{}", addr);
    println!("🔗 区块链浏览器: http://{}/explorer", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
