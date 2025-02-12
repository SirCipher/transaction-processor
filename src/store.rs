use crate::client::{Account, TransactionError, TransactionLedgerEntry};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// A deposit transaction store.
#[derive(Clone, Default, Debug)]
pub struct TransactionStore {
    log: Arc<RwLock<HashMap<u32, TransactionLedgerEntry>>>,
}

impl TransactionStore {
    /// Inserts or updates the transaction in the store.
    pub fn put_transaction(&self, transaction: TransactionLedgerEntry) {
        let inner = &mut self.log.write();
        inner.insert(transaction.transaction_id(), transaction);
    }

    /// Returns the transaction associated with `transaction_id`.
    pub fn get_transaction(
        &self,
        transaction_id: u32,
    ) -> Result<TransactionLedgerEntry, TransactionError> {
        let inner = &mut self.log.read();
        inner
            .get(&transaction_id)
            .ok_or(TransactionError::TransactionNotFound)
            .cloned()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        let inner = &mut self.log.read();
        inner.is_empty()
    }
}

/// A store containing the state of accounts.
#[derive(Clone, Default, Debug)]
pub struct AccountStore {
    accounts: Arc<RwLock<HashMap<u16, Account>>>,
}

impl AccountStore {
    /// Inserts or updates the account in the store.
    pub fn put_account(&self, account: Account) {
        let inner = &mut self.accounts.write();
        inner.insert(account.client_id(), account);
    }

    /// Returns the account associated with `account_id`.
    pub fn get_account(&self, account_id: u16) -> Option<Account> {
        let inner = &mut self.accounts.read();
        inner.get(&account_id).cloned()
    }

    /// Executes `f` against every account in this store.
    pub fn for_each(&self, mut f: impl FnMut(&Account)) {
        let inner = &mut self.accounts.read();
        inner.iter().for_each(|(_, account)| f(account));
    }
}
