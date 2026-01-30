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
        //silent failure if not enough funds
        if account.available >= amount {
            account.available -= amount;
        }
    }

    fn dispute(&mut self, record: TransactionRecord) {
        let deposit = match self.deposits.get_mut(&record.tx_id) {
            Some(d) => d,
            None => return,
        };
        
        if deposit.client_id != record.client_id || deposit.status != DepositStatus::Normal {
            return;
        }

        let account = match self.accounts.get_mut(&record.client_id) {
            Some(a) => a,
            None => return,
        };

        account.available -= deposit.amount;
        account.held += deposit.amount;
        deposit.status = DepositStatus::Disputed;
    }

    fn resolve(&mut self, record: TransactionRecord) {
        let deposit = match self.deposits.get_mut(&record.tx_id) {
            Some(d) => d,
            None => return,
        };

        if deposit.client_id != record.client_id || deposit.status != DepositStatus::Disputed {
            return;
        }

        let account = match self.accounts.get_mut(&record.client_id) {
            Some(a) => a,
            None => return,
        };

        account.held -= deposit.amount;
        account.available += deposit.amount;
        deposit.status = DepositStatus::Normal;
    }

    fn chargeback(&mut self, record: TransactionRecord) {
        let deposit = match self.deposits.get_mut(&record.tx_id) {
            Some(d) => d,
            None => return,
        };

        if deposit.client_id != record.client_id || deposit.status != DepositStatus::Disputed {
            return;
        }

        let account = match self.accounts.get_mut(&record.client_id) {
            Some(a) => a,
            None => return,
        };

        account.held -= deposit.amount;
        account.locked = true;
        deposit.status = DepositStatus::ChargedBack;
    }
}
