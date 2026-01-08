use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use eyre::Result;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    // Arbitrum Sepolia 测试网 RPC URL
    let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";

    // 使用 ProviderBuilder 构建 HTTP Provider（默认使用 Ethereum 网络类型即可，Alloy 会自动适配）
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);


    let address = "0x8EdA948B537A32C0AeAEA8c90d41cD9982972e14";
    let result = get_eth_balance(&provider, address).await.unwrap();
    println!("地址 {} 的余额: {} wei", address, result);
    Ok(())
}

pub async fn get_eth_balance<T: Provider>(
    provider: &T,
    address_str: &str,
) -> Result<String> {
    // 解析地址
    let address = Address::from_str(address_str).unwrap();

    // 查询余额（wei）
    let balance_wei: U256 = provider.get_balance(address).await?;

    // 使用 alloy 内置工具精确转换为 ETH 字符串
    Ok(alloy::primitives::utils::format_ether(balance_wei))
}