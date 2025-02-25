use alloy::primitives::Address;
use alloy::primitives::B256;
use alloy::primitives::U256;
use alloy_network::EthereumWallet;
use alloy_network::TransactionBuilder;
use alloy_provider::Provider;
use alloy_provider::ProviderBuilder;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_macro::sol;
use anyhow::Result;
use anyhow::{anyhow, Context};
use hex_literal::hex;
use reqwest::Url;

sol!(
    #[sol(rpc)]
    AvailBridgeTokenContract,
    "availtokencontract.json"
);

#[tokio::main]
async fn main() -> Result<()> {
    let from_seed = "<your seed here in hex>";
    let to_address: Address = hex!("37d4086c0755e54B4e5048Af2c402C8c3b484Ce5").into();
    let contract_addr = "0xf50F2B4D58ce2A24b62e480d795A974eD0f77A58";
    let url = "https://sepolia.base.org";

    let amount = 10_000_000_000_000_000_000u128; // 10 Avail
    transfer_erc20_avail(url, &contract_addr, &from_seed, &to_address, amount).await?;
    Ok(())
}

pub async fn transfer_erc20_avail(
    url: &str,
    token_contract_address: &str,
    sender_seed: &str,
    destination: &Address,
    amount: u128,
) -> Result<(u128, B256)> {
    let signer = sender_seed
        .parse::<alloy_signer_local::PrivateKeySigner>()
        .unwrap();
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .on_http(Url::parse(&url).context("failed parsing eth url")?);

    let token_contract = AvailBridgeTokenContract::new(
        token_contract_address
            .parse()
            .context("unable to parse token contract address")?,
        &provider,
    );

    let tx_hash = token_contract
        .transfer(destination.0.into(), U256::from(amount))
        .send()
        .await
        .context("Failed sending transaction")?
        .watch()
        .await
        .context("Failed watching the transaction")?;

    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await?
        .ok_or(anyhow!("No receipt!"))?;

    println!("Receipt: {receipt:?}");

    Ok((amount, tx_hash.into()))
}

pub async fn transfer_eth(
    url: &str,
    sender_seed: &str,
    destination: &Address,
    amount: u128,
) -> Result<(u128, B256)> {
    let signer = sender_seed
        .parse::<alloy_signer_local::PrivateKeySigner>()
        .unwrap();
    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer.clone()))
        .on_http(Url::parse(&url).context("failed parsing eth url")?);

    let p = TransactionRequest::default()
        .with_from(signer.address())
        .with_to(*destination)
        .with_value(U256::from(amount));

    let tx_hash = provider.send_transaction(p).await?.watch().await?;

    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await?
        .ok_or(anyhow!("No receipt!"))?;

    println!("Receipt: {receipt:?}");

    Ok((amount, tx_hash.into()))
}
