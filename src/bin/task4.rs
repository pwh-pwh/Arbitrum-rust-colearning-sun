use alloy::{
    network::TransactionBuilder,
    primitives::{address, utils::{format_ether, Unit}, Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use std::env;
use std::error::Error;
use std::str::FromStr;

// Arbitrum Sepolia 配置
const RPC_URL: &str = "https://sepolia-rollup.arbitrum.io/rpc";
const CHAIN_ID: u64 = 421614;

// 基础转账固定 Gas Limit
const GAS_LIMIT: u64 = 21_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // ==================== 1. 从环境变量读取配置（安全） ====================
    let private_key_hex = env::var("PRIVATE_KEY")
        .expect("请设置环境变量 PRIVATE_KEY (格式: 0x...)");

    let to_address_str = "0x6CF383b4D0c53e13B54742a14e12684936644707".to_string();

    let amount_eth_str = "0.001".to_string();

    // ==================== 2. 解析与校验 ====================
    let signer: PrivateKeySigner = private_key_hex.parse()?;
    let from_address: Address = signer.address();

    let to_address: Address = Address::from_str(&to_address_str)
        .map_err(|_| "无效的接收地址格式")?;

    let amount_eth: f64 = amount_eth_str.parse()
        .map_err(|_| "AMOUNT_ETH 必须是有效的数字")?;

    let value_wei: U256 = Unit::ETHER.wei() * U256::from(amount_eth as u128);

    println!("发送者地址: {from_address}");
    println!("接收者地址: {to_address}");
    println!("转账金额: {} ETH ({} wei)", amount_eth, value_wei);

    // ==================== 3. 构建带钱包的 Provider ====================
    let provider = ProviderBuilder::new()
        .with_chain_id(CHAIN_ID)           // 显式设置 chain_id，避免签名错误
        .wallet(signer.clone())            // 注入 signer
        .connect_http(RPC_URL.parse()?);

    // ==================== 4. 获取推荐 Gas 价格（EIP-1559） ====================
    let priority_fee: u128 = provider.get_max_priority_fee_per_gas().await?;
    let latest_block = provider
        .get_block_by_number(alloy::eips::BlockNumberOrTag::Latest)
        .await?
        .ok_or("无法获取最新区块")?;

    let base_fee = latest_block.header.base_fee_per_gas.unwrap_or(u64::try_from(U256::ZERO)?);

    // Arbitrum 推荐：max_fee = base_fee × 2 + priority_fee + 小缓冲
    let max_fee_per_gas = U256::from(base_fee
        .checked_mul(u64::try_from(U256::from(2u64))?)
        .unwrap_or(base_fee))
        + U256::from(priority_fee)  // ✅ 关键：这里转换类型
        + U256::from(100_000_000u64); // 0.1 gwei buffer

    let max_priority_fee_per_gas = U256::from(priority_fee);

    println!("Gas 设置:");

    println!("  max_priority_fee_per_gas: {} wei", max_priority_fee_per_gas);

    // ==================== 5. 余额检查 ====================
    let balance = provider.get_balance(from_address).await?;
    let estimated_fee = max_fee_per_gas * U256::from(GAS_LIMIT);

    if balance < value_wei + estimated_fee {
        return Err(format!(
            "余额不足！当前余额: {} ETH，需要: {} ETH (含 Gas 费)",
            format_ether(balance),
            format_ether(value_wei + estimated_fee)
        ).into());
    }

    println!("余额充足: {} ETH", format_ether(balance));

    // ==================== 6. 构建并发送交易 ====================
    let tx = TransactionRequest::default()
        .with_to(to_address)
        .with_value(value_wei)
        .with_gas_limit(GAS_LIMIT)
        .with_max_fee_per_gas(u128::try_from(max_fee_per_gas)?)
        .with_max_priority_fee_per_gas(u128::try_from(max_priority_fee_per_gas)?);

    let pending_tx = provider.send_transaction(tx).await?;

    println!("交易已广播！Tx Hash: {}", pending_tx.tx_hash());

    // ==================== 7. 等待确认并获取 receipt ====================
    let receipt = pending_tx
        .get_receipt()
        .await?;

    if receipt.status() {
        println!(
            "✅ 转账成功！区块号: {:?}",
            receipt.block_number.expect("无区块号")
        );
        println!("   转账 {} ETH 给 {to_address}", amount_eth);
    } else {
        println!("❌ 交易失败（revert）");
    }

    Ok(())
}