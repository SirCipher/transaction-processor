use crate::client::account::DisputeState;
use crate::client::balance::InvalidOperation;
use crate::client::{Account, TransactionError, TransactionLedgerEntry};
use crate::store::TransactionStore;
use crate::transaction::Transaction;

#[test]
fn invalid_client_id() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert_eq!(
        client.execute_transaction(Transaction::deposit(2, 0, 13), &store),
        Err(TransactionError::MismatchedClientId)
    );
    assert!(store.is_empty());
}

#[test]
fn deposit() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert!(client
        .execute_transaction(Transaction::deposit(1, 0, 13), &store)
        .is_ok());

    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::NotDisputed
        ))
    )
}

#[test]
fn negative_deposit() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert_eq!(
        client.execute_transaction(Transaction::deposit(1, 0, -13), &store),
        Err(TransactionError::InvalidOperation(
            InvalidOperation::NegativeDeposit
        ))
    );
    assert!(store.is_empty());
}

#[test]
fn withdrawal() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert!(client
        .execute_transaction(Transaction::deposit(1, 0, 13), &store)
        .is_ok());
    assert!(client
        .execute_transaction(Transaction::withdrawal(1, 0, 3), &store)
        .is_ok());

    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::NotDisputed
        ))
    );

    assert_eq!(client.total_balance(), 10.into());
    assert_eq!(client.available_balance(), 10.into());
}

#[test]
fn negative_withdrawal() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert_eq!(
        client.execute_transaction(Transaction::withdrawal(1, 0, -13), &store),
        Err(TransactionError::InvalidOperation(
            InvalidOperation::NegativeWithdrawal
        ))
    );
    assert!(store.is_empty());
}

#[test]
fn insufficient_funds() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert!(client
        .execute_transaction(Transaction::deposit(1, 0, 13), &store)
        .is_ok());
    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::NotDisputed
        ))
    );
    assert_eq!(
        client.execute_transaction(Transaction::withdrawal(1, 1, 133), &store),
        Err(TransactionError::InvalidOperation(
            InvalidOperation::InsufficientFunds
        ))
    );
}

#[test]
fn dispute() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert!(client
        .execute_transaction(Transaction::deposit(1, 0, 13), &store)
        .is_ok());
    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::NotDisputed
        ))
    );

    assert!(client
        .execute_transaction(Transaction::dispute(1, 0,), &store)
        .is_ok());

    assert_eq!(client.total_balance(), 13.into());
    assert_eq!(client.available_balance(), 0.into());
    assert_eq!(client.held_balance(), 13.into());

    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::Disputed
        ))
    );
}

#[test]
fn double_dispute() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert!(client
        .execute_transaction(Transaction::deposit(1, 0, 13), &store)
        .is_ok());
    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::NotDisputed
        ))
    );

    assert!(client
        .execute_transaction(Transaction::dispute(1, 0,), &store)
        .is_ok());

    assert_eq!(client.total_balance(), 13.into());
    assert_eq!(client.available_balance(), 0.into());
    assert_eq!(client.held_balance(), 13.into());

    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::Disputed
        ))
    );

    assert!(client
        .execute_transaction(Transaction::dispute(1, 0,), &store)
        .is_ok());

    assert_eq!(client.total_balance(), 13.into());
    assert_eq!(client.available_balance(), 0.into());
    assert_eq!(client.held_balance(), 13.into());

    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::Disputed
        ))
    );
}

#[test]
fn resolve() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert!(client
        .execute_transaction(Transaction::deposit(1, 0, 13), &store)
        .is_ok());
    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::NotDisputed
        ))
    );

    assert!(client
        .execute_transaction(Transaction::dispute(1, 0,), &store)
        .is_ok());

    assert_eq!(client.total_balance(), 13.into());
    assert_eq!(client.available_balance(), 0.into());
    assert_eq!(client.held_balance(), 13.into());

    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::Disputed
        ))
    );

    assert!(client
        .execute_transaction(Transaction::resolve(1, 0,), &store)
        .is_ok());

    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::Resolved
        ))
    );

    assert_eq!(client.total_balance(), 13.into());
    assert_eq!(client.available_balance(), 13.into());
    assert_eq!(client.held_balance(), 0.into());
}

#[test]
fn dispute_unknown_transaction() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert_eq!(
        client.execute_transaction(Transaction::dispute(1, 0,), &store),
        Err(TransactionError::TransactionNotFound)
    );
}

#[test]
fn chargeback() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert!(client
        .execute_transaction(Transaction::deposit(1, 0, 13), &store)
        .is_ok());
    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::NotDisputed
        ))
    );

    assert!(client
        .execute_transaction(Transaction::dispute(1, 0,), &store)
        .is_ok());

    assert_eq!(client.total_balance(), 13.into());
    assert_eq!(client.available_balance(), 0.into());
    assert_eq!(client.held_balance(), 13.into());

    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::Disputed
        ))
    );

    assert!(client
        .execute_transaction(Transaction::chargeback(1, 0,), &store)
        .is_ok());

    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::Disputed
        ))
    );

    assert_eq!(client.total_balance(), 0.into());
    assert_eq!(client.available_balance(), 0.into());
    assert_eq!(client.held_balance(), 0.into());

    assert!(client.is_frozen());
    assert_eq!(
        client.execute_transaction(Transaction::deposit(1, 0, 13), &store),
        Err(TransactionError::AccountFrozen)
    );
}

#[test]
fn overflow() {
    let store = TransactionStore::default();
    let mut client = Account::new(1);

    assert!(client
        .execute_transaction(Transaction::deposit(1, 0, 13), &store)
        .is_ok());
    assert_eq!(
        store.get_transaction(0),
        Ok(TransactionLedgerEntry::new(
            1,
            0,
            13.into(),
            DisputeState::NotDisputed
        ))
    );
    assert!(client
        .execute_transaction(Transaction::deposit(1, 0, 13), &store)
        .is_ok());
    assert_eq!(
        client.execute_transaction(Transaction::deposit(1, 0, (1 << 96) - 1u128), &store),
        Err(TransactionError::InvalidOperation(
            InvalidOperation::Overflow
        ))
    );
}
