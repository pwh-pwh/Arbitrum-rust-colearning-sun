use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::address;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Arbitrum Sepolia 测试网 RPC URL
    let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";

    // 使用 ProviderBuilder 构建 HTTP Provider（默认使用 Ethereum 网络类型即可，Alloy 会自动适配）
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);

    // 验证连接：获取最新块号
    let block_number = provider.get_block_number().await?;
    println!("Arbitrum Sepolia 当前最新块号: {}", block_number);

    // 获取链 ID（应为 421614）
    let chain_id = provider.get_chain_id().await?;
    println!("链 ID: {}", chain_id);
    if chain_id == 421614u64 {
        println!("成功确认连接到 Arbitrum Sepolia 测试网！");
    } else {
        println!("警告：链 ID 不匹配预期，可能连接到其他网络");
    }


    Ok(())
}