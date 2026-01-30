use std::collections::HashMap;

use crate::types::*;

// The payments engine. Holds all client accounts and stored deposits.
pub struct Engine {
    accounts: HashMap<ClientId, Account>,
    deposits: HashMap<TxId, DepositRecord>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            deposits: HashMap::new(),
        }
    }

    // Process a single transaction record, updating internal state.
    pub fn process(&mut self, record: TransactionRecord) {
        // Ignore locked accounts
        if let Some(account) = self.accounts.get(&record.client_id) {
            if account.locked {
                return;
            }
        }

        match record.tx_type {
            TxType::Deposit => self.deposit(record),
            TxType::Withdrawal => self.withdrawal(record),
            TxType::Dispute => self.dispute(record),
            TxType::Resolve => self.resolve(record),
            TxType::Chargeback => self.chargeback(record),
        }
    }

    // Produce the final output for every client account
    pub fn output(&self) -> Vec<AccountOutput> {
        self.accounts
            .iter()
            .map(|(&client, account)| AccountOutput {
                client,
                available: account.available,
                held: account.held,
                total: account.total(),
                locked: account.locked,
            })
            .collect()
    }

    fn deposit(&mut self, record: TransactionRecord) {
        let amount = match record.amount {
            Some(a) => a,
            None => return,
        };

        if self.deposits.contains_key(&record.tx_id) {
            return;
        }

        let account = self.accounts.entry(record.client_id).or_default();
        account.available += amount;

        self.deposits.insert(
            record.tx_id,
            DepositRecord {
                client_id: record.client_id,
                amount,
                status: DepositStatus::Normal,
            },
        );
    }

    fn withdrawal(&mut self, record: TransactionRecord) {
        let amount = match record.amount {
            Some(a) => a,
            None => return,
        };

        let account = self.accounts.entry(record.client_id).or_default();
        //not enough funds
        if account.available >= amount {
            account.available -= amount;
        }
        else{
            eprintln!(
                "Withdrawal failed: Insufficient funds client {} , tx {} 
                Available: {} , Withdrawal: {}",
                record.client_id, record.tx_id, account.available, amount)
        }
    }

    fn dispute(&mut self, record: TransactionRecord) {
        let deposit = match self.deposits.get_mut(&record.tx_id) {
            Some(d) => d,
            None => {
                eprintln!("Dispute failed: tx {} not found", record.tx_id);
                return;
            }
        };

        if deposit.client_id != record.client_id {
            eprintln!("Dispute failed: client {} does not own tx {}", record.client_id, record.tx_id);
            return;
        }

        if deposit.status != DepositStatus::Normal {
            eprintln!("Dispute failed: tx {} has status {:?}, expected Normal", record.tx_id, deposit.status);
            return;
        }

        let account = match self.accounts.get_mut(&record.client_id) {
            Some(a) => a,
            None => {
                eprintln!("Dispute failed: account {} not found", record.client_id);
                return;
            }
        };

        account.available -= deposit.amount;
        account.held += deposit.amount;
        deposit.status = DepositStatus::Disputed;
    }

    fn resolve(&mut self, record: TransactionRecord) {
        let deposit = match self.deposits.get_mut(&record.tx_id) {
            Some(d) => d,
            None => {
                eprintln!("Resolve failed: tx {} not found", record.tx_id);
                return;
            }
        };

        if deposit.client_id != record.client_id {
            eprintln!("Resolve failed: client {} does not own tx {}", record.client_id, record.tx_id);
            return;
        }

        if deposit.status != DepositStatus::Disputed {
            eprintln!("Resolve failed: tx {} has status {:?}, expected Disputed", record.tx_id, deposit.status);
            return;
        }

        let account = match self.accounts.get_mut(&record.client_id) {
            Some(a) => a,
            None => {
                eprintln!("Resolve failed: account {} not found", record.client_id);
                return;
            }
        };

        account.held -= deposit.amount;
        account.available += deposit.amount;
        deposit.status = DepositStatus::Normal;
    }

    fn chargeback(&mut self, record: TransactionRecord) {
        let deposit = match self.deposits.get_mut(&record.tx_id) {
            Some(d) => d,
            None => {
                eprintln!("Chargeback failed: tx {} not found", record.tx_id);
                return;
            }
        };

        if deposit.client_id != record.client_id {
            eprintln!("Chargeback failed: client {} does not own tx {}", record.client_id, record.tx_id);
            return;
        }

        if deposit.status != DepositStatus::Disputed {
            eprintln!("Chargeback failed: tx {} has status {:?}, expected Disputed", record.tx_id, deposit.status);
            return;
        }

        let account = match self.accounts.get_mut(&record.client_id) {
            Some(a) => a,
            None => {
                eprintln!("Chargeback failed: account {} not found", record.client_id);
                return;
            }
        };

        account.held -= deposit.amount;
        account.locked = true;
        deposit.status = DepositStatus::ChargedBack;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn tx(tx_type: TxType, client_id: ClientId, tx_id: TxId, amount: Option<rust_decimal::Decimal>) -> TransactionRecord {
        TransactionRecord { tx_type, client_id, tx_id, amount }
    }

    fn get_account(engine: &Engine, client_id: ClientId) -> &Account {
        engine.accounts.get(&client_id).expect("account not found")
    }

    #[test]
    fn resolve_returns_funds() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(20.0))));
        engine.process(tx(TxType::Dispute, 1, 1, None));
        engine.process(tx(TxType::Resolve, 1, 1, None));

        let acc = get_account(&engine, 1);
        assert_eq!(acc.available, dec!(20.0));
        assert_eq!(acc.held, dec!(0));
        assert!(!acc.locked);
    }

    #[test]
    fn chargeback_removes_held_and_locks() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(20.0))));
        engine.process(tx(TxType::Dispute, 1, 1, None));
        engine.process(tx(TxType::Chargeback, 1, 1, None));

        let acc = get_account(&engine, 1);
        assert_eq!(acc.available, dec!(0));
        assert_eq!(acc.held, dec!(0));
        assert_eq!(acc.total(), dec!(0));
        assert!(acc.locked);
    }

    //invalid tx_id
    #[test]
    fn dispute_nonexistent_tx_ignored() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(10.0))));
        engine.process(tx(TxType::Dispute, 1, 99, None));

        let acc = get_account(&engine, 1);
        assert_eq!(acc.available, dec!(10.0));
        assert_eq!(acc.held, dec!(0));
    }

    //client doesn't own tx_id
    #[test]
    fn dispute_wrong_client_ignored() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(10.0))));
        engine.process(tx(TxType::Deposit, 2, 2, Some(dec!(5.0))));
        // Client 2 tries to dispute client 1's deposit
        engine.process(tx(TxType::Dispute, 2, 1, None));

        let acc1 = get_account(&engine, 1);
        assert_eq!(acc1.available, dec!(10.0));
        assert_eq!(acc1.held, dec!(0));

        let acc2 = get_account(&engine, 2);
        assert_eq!(acc2.available, dec!(5.0));
        assert_eq!(acc2.held, dec!(0));
    }

    //attempt to resolve nondisputed tx
    #[test]
    fn resolve_without_dispute_ignored() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(10.0))));
        engine.process(tx(TxType::Resolve, 1, 1, None));

        let acc = get_account(&engine, 1);
        assert_eq!(acc.available, dec!(10.0));
        assert_eq!(acc.held, dec!(0));
    }

    #[test]
    fn chargeback_without_dispute_ignored() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(10.0))));
        engine.process(tx(TxType::Chargeback, 1, 1, None)); // not disputed

        let acc = get_account(&engine, 1);
        assert_eq!(acc.available, dec!(10.0));
        assert!(!acc.locked);
    }

    #[test]
    fn double_dispute_ignored() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(10.0))));
        engine.process(tx(TxType::Dispute, 1, 1, None));
        engine.process(tx(TxType::Dispute, 1, 1, None)); // already disputed

        let acc = get_account(&engine, 1);
        assert_eq!(acc.available, dec!(0));
        assert_eq!(acc.held, dec!(10.0));
    }

    #[test]
    fn duplicate_deposit_tx_id_ignored() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(10.0))));
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(20.0)))); // duplicate tx_id

        let acc = get_account(&engine, 1);
        // second deposit should be ignored; balance stays at 10
        assert_eq!(acc.available, dec!(10.0));
    }

    #[test]
    fn deposit_with_no_amount_ignored() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, None));

        // Account may or may not exist; if it does, balance is 0
        assert!(engine.accounts.get(&1).is_none());
    }

    #[test]
    fn withdrawal_with_no_amount_ignored() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(10.0))));
        engine.process(tx(TxType::Withdrawal, 1, 2, None));

        let acc = get_account(&engine, 1);
        assert_eq!(acc.available, dec!(10.0)); // unchanged
    }

    #[test]
    fn dispute_after_partial_withdrawal_makes_available_negative() {
        let mut engine = Engine::new();
        engine.process(tx(TxType::Deposit, 1, 1, Some(dec!(10.0))));
        engine.process(tx(TxType::Withdrawal, 1, 2, Some(dec!(6.0))));
        // available = 4, now dispute the original 10.0 deposit
        engine.process(tx(TxType::Dispute, 1, 1, None));

        let acc = get_account(&engine, 1);
        // available goes negative: 4 - 10 = -6
        assert_eq!(acc.available, dec!(-6.0));
        assert_eq!(acc.held, dec!(10.0));
        assert_eq!(acc.total(), dec!(4.0));
    }
}
