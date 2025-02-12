use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A transfer transaction model.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferTransaction {
    /// A deposit transaction.
    Deposit {
        /// The amount to deposit.
        amount: Decimal,
    },
    /// A withdrawal transaction.
    Withdrawal {
        /// The amount to withdraw.
        amount: Decimal,
    },
}

/// A disputed transaction model.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisputedTransaction {
    Dispute,
    Resolve,
    Chargeback,
}

/// A transaction discriminant.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionKind {
    /// A transfer transaction model.
    Transfer(TransferTransaction),
    /// A withdrawal transaction.
    Dispute(DisputedTransaction),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// The client ID to execute the transaction against.
    pub client_id: u16,
    /// A unique identifier representing the transaction number.
    ///
    /// If this transaction is a transfer then this is a new transaction number. If it is a
    /// dispute transaction then this represents the transaction to dispute.
    pub transaction_id: u32,
    /// A transaction discriminant.
    pub kind: TransactionKind,
}

impl Transaction {
    pub fn deposit(client_id: u16, transaction_id: u32, amount: impl Into<Decimal>) -> Transaction {
        Transaction {
            client_id,
            transaction_id,
            kind: TransactionKind::Transfer(TransferTransaction::Deposit {
                amount: amount.into(),
            }),
        }
    }

    pub fn withdrawal(
        client_id: u16,
        transaction_id: u32,
        amount: impl Into<Decimal>,
    ) -> Transaction {
        Transaction {
            client_id,
            transaction_id,
            kind: TransactionKind::Transfer(TransferTransaction::Withdrawal {
                amount: amount.into(),
            }),
        }
    }

    pub fn dispute(client_id: u16, transaction_id: u32) -> Transaction {
        Transaction {
            client_id,
            transaction_id,
            kind: TransactionKind::Dispute(DisputedTransaction::Dispute),
        }
    }

    pub fn resolve(client_id: u16, transaction_id: u32) -> Transaction {
        Transaction {
            client_id,
            transaction_id,
            kind: TransactionKind::Dispute(DisputedTransaction::Resolve),
        }
    }

    pub fn chargeback(client_id: u16, transaction_id: u32) -> Transaction {
        Transaction {
            client_id,
            transaction_id,
            kind: TransactionKind::Dispute(DisputedTransaction::Chargeback),
        }
    }
}
