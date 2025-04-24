#![allow(dead_code)]

use std::collections::VecDeque;
use arrayref::array_ref;
use raydium_amm_v3::states::{AmmConfig, PoolState, TickArrayBitmapExtension, TickArrayState};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use spl_token_2022::extension::StateWithExtensions;
use tracing::debug;
use crate::AMM_CONFIG_INDEX;
use crate::utils::{deserialize_anchor_account, get_out_put_amount_and_remaining_accounts, sqrt_price_x64_to_price};

#[derive(Debug, Clone)]
pub enum SwapDirection {
    Buy,
    Sell,
}

pub fn calculate_tick_array_accounts(
    rpc_client: &RpcClient,
    input_token: &Pubkey,
    output_token: &Pubkey,
    swap_direction: SwapDirection,
    amount_specified: u64,
) -> anyhow::Result<Vec<AccountMeta>> {
    let amm_config_key = calc_pda_amm_config_account(AMM_CONFIG_INDEX);

    let (token_0, token_1) = match swap_direction {
        // aka zero_for_one
        SwapDirection::Buy => (output_token, input_token),
        SwapDirection::Sell => (input_token, output_token),
    };

    let pool_id_account = calc_pda_pool_id_account(token_0, token_1, &amm_config_key);

    let tickarray_bitmap_extension_pubkey = TickArrayBitmapExtension::key(pool_id_account.clone());

    let load_accounts = vec![
        *input_token,
        *output_token,
        amm_config_key,
        pool_id_account,
        tickarray_bitmap_extension_pubkey,
    ];

    let rsps = rpc_client.get_multiple_accounts(&load_accounts)?;
    let [user_input_account, user_output_account, amm_config_account, pool_account, tickarray_bitmap_extension_account] =
        array_ref![rsps, 0, 5];
    let _user_input_state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(
        &user_input_account.as_ref().unwrap().data,
    )?;
    let _user_output_state = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(
        &user_output_account.as_ref().unwrap().data,
    )?;
    let amm_config: AmmConfig =
        deserialize_anchor_account(amm_config_account.as_ref().unwrap()).unwrap();
    let pool_state: PoolState = deserialize_anchor_account(pool_account.as_ref().unwrap()).unwrap();
    let tickarray_bitmap_extension =
        deserialize_anchor_account(tickarray_bitmap_extension_account.as_ref().unwrap()).unwrap();

    // from solscan "amount"
    // 0.01 SOL
    // let amount_specified = 10000000;
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

    let zero_for_one =
        input_token == &pool_state.token_mint_0 && output_token == &pool_state.token_mint_1;

    let mut tick_arrays = load_cur_and_next_five_tick_array(
        rpc_client,
        &pool_id_account,
        &pool_state,
        &tickarray_bitmap_extension,
        zero_for_one,
    )?;

    let current_price_sqrt = pool_state.sqrt_price_x64;
    let current_price = sqrt_price_x64_to_price(
        current_price_sqrt,
        pool_state.mint_decimals_0,
        pool_state.mint_decimals_1,
    );
    debug!("current_price: {}", current_price);

    let base_in = true;
    let (_other_amount_threshold, tick_array_indexs) = get_out_put_amount_and_remaining_accounts(
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

    Ok(remaining_accounts)
}

fn calc_pda_amm_config_account(config_index: i16) -> Pubkey {
    let (amm_config_key, __bump) = Pubkey::find_program_address(
        &[
            raydium_amm_v3::states::AMM_CONFIG_SEED.as_bytes(),
            &config_index.to_be_bytes(),
        ],
        &raydium_amm_v3::ID,
    );

    amm_config_key
}

fn calc_pda_pool_id_account(mint0: &Pubkey, mint1: &Pubkey, amm_config_key: &Pubkey) -> Pubkey {
    let (mint0, mint1) = if mint0 < mint1 {
        (mint0, mint1)
    } else {
        (mint1, mint0)
    };

    let (pool_id_account, __bump) = Pubkey::find_program_address(
        &[
            raydium_amm_v3::states::POOL_SEED.as_bytes(),
            amm_config_key.to_bytes().as_ref(),
            mint0.to_bytes().as_ref(),
            mint1.to_bytes().as_ref(),
        ],
        &raydium_amm_v3::ID,
    );

    pool_id_account
}

fn load_cur_and_next_five_tick_array(
    rpc_client: &RpcClient,
    pool_id_account: &Pubkey,
    pool_state: &PoolState,
    tickarray_bitmap_extension: &TickArrayBitmapExtension,
    zero_for_one: bool,
) -> anyhow::Result<VecDeque<TickArrayState>> {
    let (_, mut current_valid_tick_array_start_index) = pool_state
        .get_first_initialized_tick_array(&Some(*tickarray_bitmap_extension), zero_for_one)?;

    let mut tick_array_keys = Vec::new();
    tick_array_keys.push(
        Pubkey::find_program_address(
            &[
                raydium_amm_v3::states::TICK_ARRAY_SEED.as_bytes(),
                pool_id_account.to_bytes().as_ref(),
                &current_valid_tick_array_start_index.to_be_bytes(),
            ],
            &raydium_amm_v3::ID,
        )
            .0,
    );

    let mut max_array_size = 5;
    while max_array_size != 0 {
        let next_tick_array_index = pool_state.next_initialized_tick_array_start_index(
            &Some(*tickarray_bitmap_extension),
            current_valid_tick_array_start_index,
            zero_for_one,
        )?;

        if let Some(next_index) = next_tick_array_index {
            current_valid_tick_array_start_index = next_index;
            tick_array_keys.push(
                Pubkey::find_program_address(
                    &[
                        raydium_amm_v3::states::TICK_ARRAY_SEED.as_bytes(),
                        pool_id_account.to_bytes().as_ref(),
                        &current_valid_tick_array_start_index.to_be_bytes(),
                    ],
                    &raydium_amm_v3::ID,
                )
                    .0,
            );
            max_array_size -= 1;
        } else {
            break;
        }
    }

    let tick_array_rsps = rpc_client.get_multiple_accounts(&tick_array_keys)?;
    let mut tick_arrays = VecDeque::new();

    for tick_array in tick_array_rsps {
        if let Some(account) = tick_array {
            let tick_array_state =
                deserialize_anchor_account::<raydium_amm_v3::states::TickArrayState>(&account)?;
            tick_arrays.push_back(tick_array_state);
        } else {
            return Err(anyhow::anyhow!("Missing account data for tick array"));
        }
    }

    Ok(tick_arrays)
}



