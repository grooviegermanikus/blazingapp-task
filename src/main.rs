mod utils;
mod math;

use std::collections::VecDeque;
use std::str::FromStr;
use anchor_lang::AccountDeserialize;
use raydium_amm_v3::states::{AmmConfig, PoolState, TickArrayBitmapExtension, TickArrayState};
use itertools::Itertools;
use raydium_amm_v3::libraries::get_tick_at_sqrt_price;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use crate::utils::{get_out_put_amount_and_remaining_accounts, price_to_sqrt_price_x64, price_to_x64, sqrt_price_x64_to_price};

const AMM_CONFIG_INDEX: i16 = 4;

fn main() -> anyhow::Result<()> {
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com/");

    let amm_config_key = pda_amm_config(AMM_CONFIG_INDEX);

    let token_0 = Pubkey::from_str("ZxBon4vcf3DVcrt63fJU52ywYm9BKZC6YuXDhb3fomo").unwrap();
    let token_1 = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let pool_id_account = calc_pool_id_account(Some(token_0), Some(token_1), &amm_config_key).unwrap();
    let input_token = token_1;
    let output_token = token_0;

    let tickarray_bitmap_extension_pubkey = TickArrayBitmapExtension::key(pool_id_account.clone());


    let load_accounts = vec![
        input_token,
        output_token,
        amm_config_key,
        pool_id_account,
        tickarray_bitmap_extension_pubkey,
    ];
    let rsps = rpc_client.get_multiple_accounts(&load_accounts)?;

    let amm_config: AmmConfig = deserialize_anchor_account(rsps[2].as_ref().unwrap()).unwrap();
    let pool_state: PoolState = deserialize_anchor_account(rsps[3].as_ref().unwrap()).unwrap();
    let tickarray_bitmap_extension = deserialize_anchor_account(rsps[4].as_ref().unwrap()).unwrap();


    // from solscan "amount"
    // 0.01 SOL
    let amount_specified = 10000000;
    // let sqrt_price_limit_x64: u128 = 1190568305734560417006;
    // let sqrt_price_limit_x64: u128 =     901697932954476299104;
    // let sqrt_price_limit_x64 = price_to_sqrt_price_x64(
    //     // https://raydium.io/swap/?inputMint=sol&outputMint=ZxBon4vcf3DVcrt63fJU52ywYm9BKZC6YuXDhb3fomo
    //     2.775e6,
    //     pool_state.mint_decimals_0,
    //     pool_state.mint_decimals_1);

    // Raydium SQL UI uses "limit=0"
    // sqrt_price_limit_x64 must be smaller than current price
    let sqrt_price_limit_x64 = 0;
    let base_in = true;

    let zero_for_one = input_token == pool_state.token_mint_0
        && output_token == pool_state.token_mint_1;

    let mut tick_arrays = load_cur_and_next_five_tick_array(
        &rpc_client,
        &pool_id_account,
        &pool_state,
        &tickarray_bitmap_extension,
        zero_for_one,
    );

    let current_price_sqrt = pool_state.sqrt_price_x64;
    let current_price = sqrt_price_x64_to_price(
        current_price_sqrt,
        pool_state.mint_decimals_0,
        pool_state.mint_decimals_1);
    println!("current_price: {}", current_price);

    let (_other_amount_threshold, tick_array_indexs) =
        get_out_put_amount_and_remaining_accounts(
            amount_specified,
            Some(sqrt_price_limit_x64),
            zero_for_one,
            base_in,
            &amm_config,
            &pool_state,
            &tickarray_bitmap_extension,
            &mut tick_arrays,
        )
            .unwrap();

    let mut remaining_accounts = Vec::new();
    remaining_accounts.push(AccountMeta::new_readonly(
        tickarray_bitmap_extension_pubkey,
        false,
    ));
    let mut accounts = tick_array_indexs
        .into_iter()
        .map(|index| {
            AccountMeta::new(
                Pubkey::find_program_address(
                    &[
                        raydium_amm_v3::states::TICK_ARRAY_SEED.as_bytes(),
                        pool_id_account.to_bytes().as_ref(),
                        &index.to_be_bytes(),
                    ],
                    &raydium_amm_v3::ID,
                )
                    .0,
                false,
            )
        })
        .collect();
    remaining_accounts.append(&mut accounts);

    for acc in remaining_accounts {
        println!("- {}", acc.pubkey);

    }
    Ok(())
}


fn load_pool_account(pool_account_pubkey: &Pubkey) -> PoolState {
    let pool_account = RpcClient::new("https://api.mainnet-beta.solana.com/")
        .get_account(pool_account_pubkey).unwrap();

    let data = &pool_account.data;

    PoolState::try_deserialize(&mut data.as_slice()).expect("Pool Account")
}

fn load_bitmap_extension(pool_account_pubkey: &Pubkey) -> TickArrayBitmapExtension {

    let extension_account_pubkey = TickArrayBitmapExtension::key(pool_account_pubkey.clone());

    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com/");
    let pool_account = rpc_client
        .get_account(&extension_account_pubkey)
        .expect("Failed to fetch extension account");

    let data = &pool_account.data;

    TickArrayBitmapExtension::try_deserialize(&mut data.as_slice()).expect("TickArrayBitmapExtension Account")
}

fn calc_tick_account(pool_id_account: &Pubkey, index: i32) -> Pubkey {
    let (pda_pubkey, _bump_seed) =
        Pubkey::find_program_address(
        &[
            raydium_amm_v3::states::TICK_ARRAY_SEED.as_bytes(),
            pool_id_account.to_bytes().as_ref(),
            &index.to_be_bytes(),
        ],
        &raydium_amm_v3::ID,
    );

    pda_pubkey
}


fn calc_pool_id_account(mut mint0: Option<Pubkey>, mut mint1: Option<Pubkey>, amm_config_key: &Pubkey) -> Option<Pubkey> {

    let pool_id_account = if mint0 != None && mint1 != None {
        if mint0.unwrap() > mint1.unwrap() {
            let temp_mint = mint0;
            mint0 = mint1;
            mint1 = temp_mint;
        }
        Some(
            Pubkey::find_program_address(
                &[
                    raydium_amm_v3::states::POOL_SEED.as_bytes(),
                    amm_config_key.to_bytes().as_ref(),
                    mint0.unwrap().to_bytes().as_ref(),
                    mint1.unwrap().to_bytes().as_ref(),
                ],
                &raydium_amm_v3::ID,
            )
                .0,
        )
    } else {
        None
    };

    pool_id_account
}

pub fn deserialize_anchor_account<T: AccountDeserialize>(account: &Account) -> anyhow::Result<T> {
    let mut data: &[u8] = &account.data;
    T::try_deserialize(&mut data).map_err(Into::into)
}



fn load_cur_and_next_five_tick_array(
    rpc_client: &RpcClient,
    pool_id_account: &Pubkey,
    // pool_config: &ClientConfig,
    pool_state: &PoolState,
    tickarray_bitmap_extension: &TickArrayBitmapExtension,
    zero_for_one: bool,
) -> VecDeque<TickArrayState> {
    let (_, mut current_vaild_tick_array_start_index) = pool_state
        .get_first_initialized_tick_array(&Some(*tickarray_bitmap_extension), zero_for_one)
        .unwrap();
    let mut tick_array_keys = Vec::new();
    tick_array_keys.push(
        Pubkey::find_program_address(
            &[
                raydium_amm_v3::states::TICK_ARRAY_SEED.as_bytes(),
                pool_id_account.to_bytes().as_ref(),
                &current_vaild_tick_array_start_index.to_be_bytes(),
            ],
            &raydium_amm_v3::ID,
        )
            .0,
    );
    let mut max_array_size = 5;
    while max_array_size != 0 {
        let next_tick_array_index = pool_state
            .next_initialized_tick_array_start_index(
                &Some(*tickarray_bitmap_extension),
                current_vaild_tick_array_start_index,
                zero_for_one,
            )
            .unwrap();
        if next_tick_array_index.is_none() {
            break;
        }
        current_vaild_tick_array_start_index = next_tick_array_index.unwrap();
        tick_array_keys.push(
            Pubkey::find_program_address(
                &[
                    raydium_amm_v3::states::TICK_ARRAY_SEED.as_bytes(),
                    pool_id_account.to_bytes().as_ref(),
                    &current_vaild_tick_array_start_index.to_be_bytes(),
                ],
                &raydium_amm_v3::ID,
            )
                .0,
        );
        max_array_size -= 1;
    }
    let tick_array_rsps = rpc_client.get_multiple_accounts(&tick_array_keys).unwrap();
    let mut tick_arrays = VecDeque::new();
    for tick_array in tick_array_rsps {
        let tick_array_state =
            deserialize_anchor_account::<raydium_amm_v3::states::TickArrayState>(
                &tick_array.unwrap(),
            )
                .unwrap();
        tick_arrays.push_back(tick_array_state);
    }
    tick_arrays
}

fn pda_amm_config(config_index: i16) -> Pubkey {
    let (amm_config_key, __bump) = Pubkey::find_program_address(
        &[
            raydium_amm_v3::states::AMM_CONFIG_SEED.as_bytes(),
            &config_index.to_be_bytes(),
        ],
        &raydium_amm_v3::ID,
    );

    amm_config_key
}

#[test]
fn test_derive_amm_config() {
    // TODO amm config should be derived but did not work actually
    for config_index in 0..i16::MAX {
        let (amm_config_key, __bump) = Pubkey::find_program_address(
            &[
                raydium_amm_v3::states::AMM_CONFIG_SEED.as_bytes(),
                &config_index.to_be_bytes(),
            ],
            &raydium_amm_v3::ID,
        );
        if amm_config_key.to_string() == "9iFER3bpjf1PTTCQCfTRu17EJgvsxo9pVyA9QWwEuX4x" {
            println!("amm_config_key: {} for config_index={}", amm_config_key, config_index);
        }
    }
}
