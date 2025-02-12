use crate::client::balance::{Balance, InvalidOperation};
use crate::store::TransactionStore;
use crate::transaction::{DisputedTransaction, Transaction, TransactionKind, TransferTransaction};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Errors which may be produced when executing a transaction.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum TransactionError {
    /// The transaction could not be processed as the account has been frozen.
    #[error("Account frozen")]
    AccountFrozen,
    /// An invalid operation was attempted on the account.
    #[error(transparent)]
    InvalidOperation(#[from] InvalidOperation),
    /// The transaction's target client ID did not match the account ID that it was attempted
    /// against.
    #[error("Mismatched client ID")]
    MismatchedClientId,
    /// An attempt was made to execute a transaction against an account which referenced a
    /// transaction that has not been processed by the account.
    #[error("Transaction not found")]
    TransactionNotFound,
}

/// Account model which a transaction is executed against.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    /// The client identifier which this account models.
    client_id: u16,
    /// The available and held funds of this account.
    balance: Balance,
    /// Whether this account has been frozen. If this is true, then no further transactions may be
    /// executed by this account.
    frozen: bool,
}

impl Account {
    /// Creates a new account which references `client_id`.
    pub fn new(client_id: u16) -> Account {
        Account {
            client_id,
            balance: Balance::default(),
            frozen: false,
        }
    }

    /// Get the client ID of this account.
    pub fn client_id(&self) -> u16 {
        self.client_id
    }

    /// Returns the balance that is available to withdraw.
    pub fn available_balance(&self) -> Decimal {
        self.balance.available()
    }

    /// Returns the balance that has been held.
    pub fn held_balance(&self) -> Decimal {
        self.balance.held()
    }

    /// Returns the sum of the held and available balance.
    pub fn total_balance(&self) -> Decimal {
        self.balance.total()
    }

    /// Returns whether this account has been frozen.
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// Attempts to execute `transaction` against this account. If an error is produced while
    /// executing the transaction then the state of the account will not change and the transaction
    /// will be ignored.
    ///
    /// If `transaction` is a deposit then an entry will be persisted into `store`.
    pub fn execute_transaction(
        &mut self,
        transaction: Transaction,
        store: &TransactionStore,
    ) -> Result<(), TransactionError> {
        let Account {
            client_id,
            balance,
            frozen,
        } = self;

        if *frozen {
            return Err(TransactionError::AccountFrozen);
        } else if *client_id != transaction.client_id {
            return Err(TransactionError::MismatchedClientId);
        }

        match transaction.kind {
            TransactionKind::Transfer(TransferTransaction::Deposit { amount }) => {
                balance.deposit(amount)?;
                store.put_transaction(TransactionLedgerEntry {
                    client_id: *client_id,
                    transaction_id: transaction.transaction_id,
                    amount,
                    disputed: DisputeState::NotDisputed,
                });
                Ok(())
            }
            TransactionKind::Transfer(TransferTransaction::Withdrawal { amount }) => {
                balance.withdraw(amount)?;
                Ok(())
            }
            TransactionKind::Dispute(dispute) => {
                let mut disputed_transaction = store.get_transaction(transaction.transaction_id)?;

                match dispute {
                    DisputedTransaction::Dispute => {
                        if matches!(disputed_transaction.disputed, DisputeState::NotDisputed) {
                            disputed_transaction.disputed = DisputeState::Disputed;
                            balance.hold(disputed_transaction.amount);
                            store.put_transaction(disputed_transaction);
                        }

                        Ok(())
                    }
                    DisputedTransaction::Resolve => {
                        if matches!(disputed_transaction.disputed, DisputeState::Disputed) {
                            disputed_transaction.disputed = DisputeState::Resolved;
                            balance.release(disputed_transaction.amount);
                            store.put_transaction(disputed_transaction);
                        }

                        Ok(())
                    }
                    DisputedTransaction::Chargeback => {
                        if matches!(disputed_transaction.disputed, DisputeState::Disputed) {
                            balance.chargeback(disputed_transaction.amount);
                            *frozen = true;
                        }

                        Ok(())
                    }
                }
            }
        }
    }
}

/// The current dispute status of a deposit transaction.
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum DisputeState {
    /// The transaction has not been disputed yet.
    NotDisputed,
    /// The transaction is currently under dispute.
    Disputed,
    /// The transaction was under dispute but it has now been resolved.
    Resolved,
}

/// A deposit transaction store entry model.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TransactionLedgerEntry {
    /// The client identifier which made the transaction.
    client_id: u16,
    /// A unique identifier for the transaction.
    transaction_id: u32,
    /// The value of the deposit.
    amount: Decimal,
    /// The current dispute status of the transaction.
    disputed: DisputeState,
}

impl TransactionLedgerEntry {
    /// Constructs a new transaction ledger entry.
    pub fn new(
        client_id: u16,
        transaction_id: u32,
        amount: Decimal,
        disputed: DisputeState,
    ) -> TransactionLedgerEntry {
        TransactionLedgerEntry {
            client_id,
            transaction_id,
            amount,
            disputed,
        }
    }

    /// Returns the unique identifier for the transaction.
    pub fn transaction_id(&self) -> u32 {
        self.transaction_id
    }
}

/// A Display wrapper for printing an account's state.
pub struct AccountPrinter<'a>(&'a Account);

impl<'a> AccountPrinter<'a> {
    pub fn new(account: &'a Account) -> AccountPrinter<'a> {
        AccountPrinter(account)
    }
}

impl<'a> Display for AccountPrinter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let AccountPrinter(Account {
            client_id,
            balance,
            frozen,
        }) = self;

        write!(
            f,
            "{}, {:.4}, {:.4}, {:.4}, {}",
            client_id,
            balance.available(),
            balance.held(),
            balance.total(),
            frozen
        )
    }
}
