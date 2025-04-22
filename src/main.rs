use std::str::FromStr;
use anchor_lang::AccountDeserialize;
use raydium_amm_v3::states::TickArrayState;
use raydium_amm_v3::util::AccountLoad;
use std::borrow::BorrowMut;
use std::collections::VecDeque;
use itertools::Itertools;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

// https://solscan.io/tx/2piXisrMFFDKNvsnyVSCWhEA5u1arQ1jEmsJDXogoKn6YrYFndFsbXzh5jMvLXyZJMkR115UsbvQ1ZSSQVshMktM
fn main() -> anyhow::Result<()> {

    let b64 = include_str!("tickarrayaccount.dat");
    let mut acc_data = base64::decode(&b64).unwrap();

    // Pool: Raydium (SOL-FOMO) Market
    let pool = Pubkey::from_str("BUgyiu6zcr2DX9kV4v58otmkkxUjdmafVMXECu6CV7DC").unwrap();
    // AccountLoad::<TickArrayState>::try_from(&tick_array_upper_loader.to_account_info())?;
    // TickArrayState::initialize(1,2, &pool).unwrap()
    let tick_array = TickArrayState::try_deserialize_unchecked(&mut acc_data.as_slice()).unwrap();

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

    let minta = Pubkey::from_str("ZxBon4vcf3DVcrt63fJU52ywYm9BKZC6YuXDhb3fomo").unwrap();
    let mintb = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let pool_id_account = calc_pool_id_account(Some(minta), Some(mintb)).unwrap();

    assert_eq!(raydium_amm_v3::ID.to_string(), "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");


    let mut accounts = vec![x]
        .iter()
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
        .collect_vec();

    for acc in accounts {
        println!("acc: {:?}", acc);

    }



    Ok(())
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