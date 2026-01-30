# Simple Payments Engine

A simple transaction processing engine written in Rust. Reads a series of transactions from a CSV file, updates client accounts and outputs the final state of all client accounts as CSV to stdout.

## Design

### Architecture

- **`types.rs`** — Core data types: `TransactionRecord`, `TxType`, `DepositStatus`, `DepositRecord`, `Account`, `AccountOutput`
- **`engine.rs`** — `Engine` struct with a `HashMap<ClientId, Account>` for client state and a `HashMap<TxId, DepositRecord>` for dispute lookups
- **`main.rs`** — CLI entry point, CSV reading/writing

### Correctness
Functionality was proven through usage of transactions.csv

Unit tests for edge cases and conditions are included in `engine.rs`, a few to mention:

- **Deposit with no amount vs deposit with 0** - A deposit tx with empty amount field will not create a new account, this tx will be ignored. A deposit with value 0 will result in a new account with balance 0.
- **Double Disputes** - A dispute can only process if the tx is in 'normal' state. Duplicate disputes will be ignored.
- **Negative balances** - i.e. Disputing a deposit after withdrawing can result in a negative balance. See lines 311-322 in `engine.rs`


### Efficiency - Streaming
Transactions are streamed by row using `csv::Reader::deserialize()`. Full input fill is never loaded into memory.
The current engine is a single threaded engine. With further implementation, we would need async loops to replace the current CSV reader. HashMaps used in `engine.rs` are not thread safe in current implementation, would need to be replaced with concurrent maps.

### Safety / Error Handling
Malformed CSV rows are logged and skipped. Invalid operations are also logged and skipped (i.e. resolving without disputing, withdrawal on insufficient funds, etc)

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

## Usage

```bash
cargo build --release
cargo run -- transactions.csv > accounts.csv
```

Test Cases

```bash
cargo test
```