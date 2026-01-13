//! Example of generating code from ABI file using the `sol!` macro to interact with the contract.
 
use alloy::{primitives::address, providers::ProviderBuilder, sol};
use eyre::Result;
 
// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IWETH9,
    "abi/a.json"
);
 
#[tokio::main]
async fn main() -> Result<()> {
    // Spin up a forked Anvil node.
    // Ensure `anvil` is available in $PATH.
    let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
 
    // Create a contract instance.
    let contract = IWETH9::new(address!("0x3031a6d5d9648ba5f50f656cd4a1672e1167a34a"), provider);
    
     // 查询合约名称
    let name = contract.name().call().await?;
    println!("Contract name is {name}");

    // Call the contract, retrieve the total supply.
    let total_supply = contract.totalSupply().call().await?;
    
    println!("WETH total supply is {total_supply}");
 
    Ok(())
}