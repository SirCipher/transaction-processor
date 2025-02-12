mod account;
mod balance;
#[cfg(test)]
mod tests;

use crate::store::{AccountStore, TransactionStore};
use crate::transaction::Transaction;
use std::sync::Arc;
use tokio::sync::{mpsc, Notify};
use tokio::task::JoinHandle;

pub use account::Account;
pub use account::{AccountPrinter, TransactionError, TransactionLedgerEntry};

const CLIENT_STOPPED: &str = "Client unexpectedly terminated";
/// Transaction request channel capacity.
const TASK_BUFFER_SIZE: usize = 32;

/// A handle to an account. Transactions may be executed against the client using this model.
#[derive(Debug)]
pub struct Client {
    /// Channel to use to send a transaction to the account.
    tx: mpsc::Sender<Transaction>,
    /// Signal that the client has terminated.
    stop: Arc<Notify>,
    /// Internal join handle for the transaction processing task.
    _task: Arc<JoinHandle<()>>,
}

impl Client {
    /// Attempts to initialise this client from the store or create a new client.
    ///
    /// # Arguments:
    /// * `account_store`: store which will be used to attempt to initialise the client from.
    /// * `transaction_store`: store which deposit transactions will be persisted in.
    /// * `account_id`: a unique identifier representing the account.
    pub fn from_store(
        account_store: AccountStore,
        transaction_store: TransactionStore,
        account_id: u16,
    ) -> Client {
        let account = account_store
            .get_account(account_id)
            .unwrap_or_else(|| Account::new(account_id));
        let (tx, rx) = mpsc::channel(TASK_BUFFER_SIZE);
        let stop = Arc::new(Notify::new());
        let task = Arc::new(tokio::spawn(run_client(
            rx,
            account_store,
            transaction_store,
            account,
            stop.clone(),
        )));

        Client {
            tx,
            _task: task,
            stop,
        }
    }

    /// Executes `transaction` against this account.
    ///
    /// # Errors:
    /// If there is an error executing the transaction then a log message will be emitted and the
    /// account's state will not have changed and the transaction will not be persisted.
    pub async fn execute_transaction(&self, transaction: Transaction) {
        self.tx.send(transaction).await.expect(CLIENT_STOPPED);
    }

    /// Gracefully shuts down this client after all pending transactions have been processed.
    pub async fn stop(self) {
        let Client { tx, _task, stop } = self;
        // Drop the sender.
        drop(tx);
        // Wait for all the pending entries to have been processed before exiting.
        stop.notified().await;
    }
}

async fn run_client(
    mut requests: mpsc::Receiver<Transaction>,
    account_store: AccountStore,
    transaction_store: TransactionStore,
    mut account: Account,
    stopped: Arc<Notify>,
) {
    while let Some(transaction) = requests.recv().await {
        let _result = account.execute_transaction(transaction, &transaction_store);
        account_store.put_account(account.clone());
    }

    // Notify any pending waiter that we have finished processing the transactions.
    stopped.notify_one();
}
