# Simple Payments Engine

A simple transaction processing engine written in Rust. Reads a series of transactions from a CSV file, updates client accounts and outputs the final state of all client accounts as CSV to stdout.

## Usage

```bash
cargo build --release
cargo run -- transactions.csv > accounts.csv
```

## Transaction Types

| Type | Effect |
|------|--------|
| **Deposit** | Increases `available` and `total` |
| **Withdrawal** | Decreases `available` and `total` (fails silently if insufficient funds) |
| **Dispute** | Moves disputed amount from `available` to `held` (`total` unchanged) |
| **Resolve** | Moves held amount back from `held` to `available` (`total` unchanged) |
| **Chargeback** | Removes held amount from `held` and `total`, freezes the account |


## Assumptions

- **Only deposits can be disputed.** The instructions state that a dispute decreases available funds and increases held funds. Applying this to a withdrawal would double the withdrawal rather than reversing it, therefore only deposits are disputable.
- **Locked accounts reject all further transactions.** Once a chargeback freezes an account, nothing else can be processed for this account. Account is locked indefinitely.
- **Duplicate transaction IDs are rejected.** The instructions state tx IDs are globally unique. If a duplicate tx ID appears, the second occurrence is ignored to prevent state corruption.
- **Client ID must match on dispute/resolve/chargeback.** A dispute referencing a tx that belongs to a different client is ignored.

