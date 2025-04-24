mod math;
mod utils;
mod tick_accounts_utils;

use solana_program::pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use std::str::FromStr;
use crate::tick_accounts_utils::{calculate_tick_array_accounts, SwapDirection};

const AMM_CONFIG_INDEX: i16 = 4;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com/");

    let direction = SwapDirection::Buy;
    let input_token = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    // FOMO
    let output_token = Pubkey::from_str("ZxBon4vcf3DVcrt63fJU52ywYm9BKZC6YuXDhb3fomo").unwrap();
    // 0.01 SOL
    let amount = 10000000;

    let tick_array_accounts =
        calculate_tick_array_accounts(
            &rpc_client,
            &input_token, &output_token,
            direction, amount
        )?;

    println!("Swapping (Buy) {} for {}", output_token, input_token);
    println!("Tick Array Accounts involved:");
    for acc in tick_array_accounts {
        println!("- {}", acc.pubkey);
    }

    Ok(())
}
