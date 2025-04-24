# About

This project demonstrates how calculate the tick-array accounts which need to build Raydium Swap transaction.

The tick-arrays contain the price ranges (i.e. the concentrated liquidity) for the respective swap pool.

The provided example works on the FOMO-SOL (see [Raydium FOMO-SOL Pool](https://raydium.io/swap/?inputMint=sol&outputMint=ZxBon4vcf3DVcrt63fJU52ywYm9BKZC6YuXDhb3fomo))

# Run
```bash
cargo run --bin blazingapp-task
```

Output:
```
Swapping (Buy) ZxBon4vcf3DVcrt63fJU52ywYm9BKZC6YuXDhb3fomo for So11111111111111111111111111111111111111112
Tick Array Accounts involved:
- Hm6nyjG8uJzMmirVMhX2bp8LbPCSu9QRpMvPkNWvzShZ
- EDfGjEhtBZaULHtZajXnhEvz9qjmeT7JELG8abXwLyn7

```

# Open Issues
* Raydium dependency should be tagged to make build more reproducible
* Sell is not tested and might not work