use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::{U256, utils::format_ether};
use std::str::FromStr;

/// 基础 ETH 转账所需的 Gas 限额（EVM 标准值）
const BASIC_TRANSFER_GAS_LIMIT: u64 = 21_000;

/// 将 U256 (wei) 转换为 gwei 并格式化为可读字符串（保留 4 位小数）
fn wei_to_gwei_str(wei: U256) -> String {
    let gwei = wei / U256::from(1_000_000_000u128); // 整数部分 gwei
    let remainder = wei % U256::from(1_000_000_000u128);
    let fractional = remainder * U256::from(10_000u128) / U256::from(1_000_000_000u128); // 保留 4 位小数
    format!("{}.{:04}", gwei, fractional)
}

/// 获取当前 Gas 价格（EIP-1559 模式）
/// 返回 (max_fee_per_gas, max_priority_fee_per_gas) 单位：wei
pub async fn get_gas_price<T: Provider>(provider: &T) -> Result<(U256, U256), Box<dyn std::error::Error>> {
    // 获取建议的 priority fee（小费）
    let priority_fee = provider.get_max_priority_fee_per_gas().await?;

    // 获取最新区块的 base fee
    let latest_block = provider
        .get_block_by_number(alloy::eips::BlockNumberOrTag::Latest)
        .await?
        .ok_or("未能获取最新区块")?;

    let base_fee_per_gas = latest_block.header.base_fee_per_gas.unwrap_or(u64::try_from(U256::ZERO)?);

    // Arbitrum 上 base fee 通常很低，推荐设置 max_fee = base_fee * 2 + priority_fee
    // 这里保守策略：base_fee * 2 + priority_fee + 0.1 gwei buffer
    let max_fee_per_gas = U256::from(base_fee_per_gas
        .checked_mul(u64::try_from(U256::from(2u64))?)
        .unwrap_or(base_fee_per_gas))
        + U256::from(priority_fee)  // ✅ 关键：这里转换类型
        + U256::from(100_000_000u64); // 0.1 gwei buffer

    Ok((max_fee_per_gas, U256::try_from(priority_fee)?))
}

/// 获取基础 ETH 转账的 Gas 限额（固定 21000）
pub const fn get_basic_transfer_gas_limit() -> u64 {
    BASIC_TRANSFER_GAS_LIMIT
}

/// 计算预估转账 Gas 费用
/// 返回 (费用 in wei, 费用 in ETH 字符串)
pub fn estimate_transfer_fee(max_fee_per_gas: U256, gas_limit: u64) -> (U256, String) {
    let fee_wei = max_fee_per_gas * U256::from(gas_limit);
    let fee_eth = format_ether(fee_wei);
    (fee_wei, fee_eth)
}

/// 综合函数：一键获取并计算预估转账费用
pub async fn get_estimated_transfer_fee<T: Provider>(
    provider: &T,
) -> Result<(String, U256), Box<dyn std::error::Error>> {
    let (max_fee_per_gas, priority_fee) = get_gas_price(provider).await?;
    let gas_limit = get_basic_transfer_gas_limit();

    let (fee_wei, fee_eth) = estimate_transfer_fee(max_fee_per_gas, gas_limit);

    println!("当前 Gas 信息 (Arbitrum Sepolia):");
    println!("  Max Fee per Gas:     {} wei ({} gwei)", max_fee_per_gas, wei_to_gwei_str(max_fee_per_gas));
    println!("  Priority Fee:        {} wei ({} gwei)", priority_fee, wei_to_gwei_str(priority_fee));
    println!("  Gas Limit:           {}", gas_limit);
    println!("  预估转账费用:         {} ETH ({} wei)", fee_eth, fee_wei);

    Ok((fee_eth, fee_wei))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Arbitrum Sepolia 测试网公共 RPC
    let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";

    // 构建 Provider
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);

    // 一键获取预估费用
    let (estimated_eth, estimated_wei) = get_estimated_transfer_fee(&provider).await?;

    println!("\n=== 最终结果 ===");
    println!("预估单次普通 ETH 转账费用 ≈ {} ETH", estimated_eth);
    println!("（对应 {} wei）", estimated_wei);

    Ok(())
}