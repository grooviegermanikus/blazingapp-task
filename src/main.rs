use std::str::FromStr;
use anchor_lang::AccountDeserialize;
use raydium_amm_v3::states::{PoolState, TickArrayBitmapExtension, TickArrayState};
use itertools::Itertools;
use raydium_amm_v3::libraries::get_tick_at_sqrt_price;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;

// https://solscan.io/tx/2piXisrMFFDKNvsnyVSCWhEA5u1arQ1jEmsJDXogoKn6YrYFndFsbXzh5jMvLXyZJMkR115UsbvQ1ZSSQVshMktM
fn main() -> anyhow::Result<()> {


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