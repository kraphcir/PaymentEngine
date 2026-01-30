use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub type ClientId = u16;
pub type TxId = u32;

// Transaction type parsed from the CSV `type` column.
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

// A raw transaction record deserialized from a CSV row.
#[derive(Debug, Deserialize)]
pub struct TransactionRecord {
    #[serde(rename = "type")]
    pub tx_type: TxType,
    #[serde(rename = "client")]
    pub client_id: ClientId,
    #[serde(rename = "tx")]
    pub tx_id: TxId,
    pub amount: Option<Decimal>,
}

// Tracks the lifecycle of a deposit through the dispute process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepositStatus {
    Normal,
    Disputed,
    ChargedBack,
}

// A stored deposit, retained so disputes can reference it by tx ID.
#[derive(Debug)]
pub struct DepositRecord {
    pub client_id: ClientId,
    pub amount: Decimal,
    pub status: DepositStatus,
}

// A single client's account
#[derive(Debug)]
pub struct Account {
    pub available: Decimal,
    pub held: Decimal,
    pub locked: bool,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
        }
    }
}

impl Account {
    pub fn total(&self) -> Decimal {
        self.available + self.held
    }
}

// Output row written to stdout
#[derive(Debug, Serialize)]
pub struct AccountOutput {
    pub client: ClientId,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}
