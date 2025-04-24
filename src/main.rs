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

fn main() -> anyhow::Result<()> {
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com/");

    // let amm_config_key = Pubkey::from_str("9iFER3bpjf1PTTCQCfTRu17EJgvsxo9pVyA9QWwEuX4x").unwrap();

    let token_0 = Pubkey::from_str("ZxBon4vcf3DVcrt63fJU52ywYm9BKZC6YuXDhb3fomo").unwrap();
    let token_1 = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let pool_id_account = calc_pool_id_account(Some(token_0), Some(token_1)).unwrap();
    let input_token = token_1;
    let output_token = token_0;

    let tickarray_bitmap_extension_pubkey = TickArrayBitmapExtension::key(pool_id_account.clone());


    let load_accounts = vec![
        input_token,
        output_token,
        Pubkey::new_unique(),// amm_config_key, not used
        pool_id_account,
        tickarray_bitmap_extension_pubkey,
    ];
    let rsps = rpc_client.get_multiple_accounts(&load_accounts)?;

    let pool_state: PoolState = deserialize_anchor_account(rsps[3].as_ref().unwrap()).unwrap();
    let tickarray_bitmap_extension = deserialize_anchor_account(rsps[4].as_ref().unwrap()).unwrap();

    let amm_config: AmmConfig = deserialize_anchor_account(&rpc_client.get_account(&pool_state.amm_config).unwrap()).unwrap();



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
    let sqrt_price_limit_x64 = 0;
    // let sqrt_price_limit_x64: u128 =     price_to_x64();
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

    let (mut other_amount_threshold, tick_array_indexs) =
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
    // println!(
    //     "amount:{}, other_amount_threshold:{}",
    //     amount, other_amount_threshold
    // );


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

// https://solscan.io/tx/2piXisrMFFDKNvsnyVSCWhEA5u1arQ1jEmsJDXogoKn6YrYFndFsbXzh5jMvLXyZJMkR115UsbvQ1ZSSQVshMktM
fn main2() -> anyhow::Result<()> {


    let b64 = include_str!("tickarrayaccount.dat");
    let mut acc_data = base64::decode(&b64).unwrap();

    // AccountLoad::<TickArrayState>::try_from(&tick_array_upper_loader.to_account_info())?;
    // TickArrayState::initialize(1,2, &pool).unwrap()
    let mut tick_array = TickArrayState::try_deserialize_unchecked(&mut acc_data.as_slice()).unwrap();

    println!("tick_array: {:?}", tick_array.ticks.len());

    tick_array.ticks.iter()
        .filter(|tick| {
            tick.tick > 0
        })
        .for_each(|tick| {
        println!("tick: {:?}", tick);
    });

    let x = tick_array.start_tick_index;
    println!("tick_array.start_tick_index: {}", x);

    assert_eq!(raydium_amm_v3::ID.to_string(), "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");

    // FOMO for SOL
    let zero_for_one = true;
    let token_0 = Pubkey::from_str("ZxBon4vcf3DVcrt63fJU52ywYm9BKZC6YuXDhb3fomo").unwrap();
    let token_1 = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let pool_id_account = calc_pool_id_account(Some(token_0), Some(token_1)).unwrap();
    // Pool: Raydium (SOL-FOMO) Market
    let pool_sol_fomo = Pubkey::from_str("BUgyiu6zcr2DX9kV4v58otmkkxUjdmafVMXECu6CV7DC").unwrap();
    assert_eq!(pool_id_account, pool_sol_fomo);

    let pool_state = load_pool_account(&pool_id_account);
    let tick_spacing = pool_state.tick_spacing;
    println!("pool_state.tick_spacing: {}", tick_spacing);
    // check if we need this
    let tick_current = pool_state.tick_current;
    println!("pool_state.tick_current: {}", tick_current);
    let tick_array_bitmap = pool_state.tick_array_bitmap;
    println!("pool_state.tick_array_bitmap: {:?}", tick_array_bitmap);

    let array_bitmap_extension = load_bitmap_extension(&pool_id_account);


    let tick_acc = calc_tick_account(&pool_id_account, tick_current);
    println!("tick_acc: {:?}", tick_acc);

    // Taken from solscan -> sqrtPriceLimitX64
    let tick_price = get_tick_at_sqrt_price(1190568305734560417006).unwrap();
    println!("tick_price: {:?}", tick_price);


    let positive_tick_array_bitmap = array_bitmap_extension.positive_tick_array_bitmap;
    let negative_tick_array_bitmap = array_bitmap_extension.negative_tick_array_bitmap;
    println!("positive_tick_array_bitmap: {:?}", positive_tick_array_bitmap);
    println!("negative_tick_array_bitmap: {:?}", negative_tick_array_bitmap);

    let mut accounts_idx = (x..x+10000)
        .map(|index| {
            let acc = AccountMeta::new(
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
            );
            (acc, index)
        })
        .collect_vec();

    for (acc, idx) in accounts_idx {
        // println!("acc: {:?}", acc);

        // 57660
        // let tick_array_account = Pubkey::from_str("4SAPjPBDdkcWLdvU194PSbsD3KDJ2wgkzFxemT6XZcRe").unwrap();
        // 60780
        let rem1 = Pubkey::from_str("EDfGjEhtBZaULHtZajXnhEvz9qjmeT7JELG8abXwLyn7").unwrap();
        if acc.pubkey == rem1 {
            println!("FOUND: {:?} {}", acc, idx);
        }


    }

    // EDfGjEhtBZaULHtZajXnhEvz9qjmeT7JELG8abXwLyn7

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

fn calc_pool_id_account(mut mint0: Option<Pubkey>, mut mint1: Option<Pubkey>) -> Option<Pubkey> {

    //Amm Config
    let amm_config_key = Pubkey::from_str("9iFER3bpjf1PTTCQCfTRu17EJgvsxo9pVyA9QWwEuX4x").unwrap();

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