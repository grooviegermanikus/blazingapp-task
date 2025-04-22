use std::str::FromStr;
use anchor_lang::AccountDeserialize;
use raydium_amm_v3::states::TickArrayState;
use raydium_amm_v3::util::AccountLoad;
use std::borrow::BorrowMut;
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


    Ok(())
}

