use crate::transaction::{DisputedTransaction, Transaction, TransactionKind, TransferTransaction};
use csv::{ReaderBuilder, Trim};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::io::Read;
use thiserror::Error;
use tokio::sync::mpsc;

#[cfg(test)]
mod tests;

/// CSV read errors which may occur.
#[derive(Error, Debug)]
pub enum ReaderError {
    /// The consumer task channel has been dropped.
    #[error("Reader forward channel dropped")]
    ChannelDropped,
    /// Failed to parse the input CSV record's amount into a transaction.
    #[error(transparent)]
    /// CSV crate failed to read a CSV record.
    Amount(#[from] AmountParseError),
    #[error(transparent)]
    Csv(csv::Error),
}

/// CSV reader task which will read from `read`, deserialize transactions and sink them into
/// `sender`.
///
/// # Errors
/// This task will terminate at the first record which it fails to parse and return a `ReaderError`.
pub async fn reader_task(
    path: impl Read,
    sender: mpsc::Sender<Transaction>,
) -> Result<(), ReaderError> {
    let reader = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .has_headers(true)
        .from_reader(path)
        .into_deserialize::<CsvTransaction>();

    for parse_result in reader {
        match parse_result {
            Ok(csv_tx) => {
                let tx = Transaction::try_from(csv_tx)?;
                if sender.send(tx).await.is_err() {
                    return Err(ReaderError::ChannelDropped);
                }
            }
            Err(e) => return Err(ReaderError::Csv(e)),
        }
    }

    Ok(())
}

/// Model of a CSV record which will be deserialized before being parsed into a Transaction.
#[derive(Serialize, Deserialize, Debug)]
pub struct CsvTransaction {
    /// The type of transaction.
    #[serde(rename = "type")]
    tx_type: TransactionType,
    /// The client's unique identifier.
    client: u16,
    /// The referenced transaction number.
    tx: u32,
    /// An associated deposit or withdrawal amount.
    amount: Option<Decimal>,
}

/// The type of transaction.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Errors produced when parsing an 'amount' associated with a CSV record.
#[derive(Error, Debug)]
pub enum AmountParseError {
    /// Expected no `amount` column to be present in the record as this was not a transfer
    /// transaction.
    #[error("Expected no amount to be provided")]
    ExpectedNoAmount,
    /// Expected an `amount` column to be present in the record as this was a transfer transaction.
    #[error("Expected an amount to be provided")]
    ExpectedAnAmount,
    /// Failed to parse the record's amount into a Decimal.
    #[error(transparent)]
    Invalid(#[from] rust_decimal::Error),
}

impl TryFrom<CsvTransaction> for Transaction {
    type Error = AmountParseError;

    fn try_from(value: CsvTransaction) -> Result<Self, Self::Error> {
        let CsvTransaction {
            tx_type,
            client,
            tx,
            amount,
        } = value;
        match tx_type {
            TransactionType::Deposit => {
                let amount = amount.ok_or(AmountParseError::ExpectedAnAmount)?;
                Ok(Transaction::deposit(client, tx, amount))
            }
            TransactionType::Withdrawal => {
                let amount = amount.ok_or(AmountParseError::ExpectedAnAmount)?;
                Ok(Transaction::withdrawal(client, tx, amount))
            }
            TransactionType::Dispute => {
                if amount.is_some() {
                    return Err(AmountParseError::ExpectedNoAmount);
                }
                Ok(Transaction::dispute(client, tx))
            }
            TransactionType::Resolve => {
                if amount.is_some() {
                    return Err(AmountParseError::ExpectedNoAmount);
                }
                Ok(Transaction::resolve(client, tx))
            }
            TransactionType::Chargeback => {
                if amount.is_some() {
                    return Err(AmountParseError::ExpectedNoAmount);
                }
                Ok(Transaction::chargeback(client, tx))
            }
        }
    }
}

impl From<Transaction> for CsvTransaction {
    fn from(value: Transaction) -> Self {
        let Transaction {
            client_id,
            transaction_id,
            kind,
        } = value;
        match kind {
            TransactionKind::Transfer(TransferTransaction::Withdrawal { amount }) => {
                CsvTransaction {
                    tx_type: TransactionType::Withdrawal,
                    client: client_id,
                    tx: transaction_id,
                    amount: Some(amount),
                }
            }
            TransactionKind::Transfer(TransferTransaction::Deposit { amount }) => CsvTransaction {
                tx_type: TransactionType::Deposit,
                client: client_id,
                tx: transaction_id,
                amount: Some(amount),
            },
            TransactionKind::Dispute(DisputedTransaction::Dispute) => CsvTransaction {
                tx_type: TransactionType::Dispute,
                client: client_id,
                tx: transaction_id,
                amount: None,
            },
            TransactionKind::Dispute(DisputedTransaction::Chargeback) => CsvTransaction {
                tx_type: TransactionType::Chargeback,
                client: client_id,
                tx: transaction_id,
                amount: None,
            },
            TransactionKind::Dispute(DisputedTransaction::Resolve) => CsvTransaction {
                tx_type: TransactionType::Resolve,
                client: client_id,
                tx: transaction_id,
                amount: None,
            },
        }
    }
}
