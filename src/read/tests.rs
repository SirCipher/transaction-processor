use crate::read::reader_task;
use crate::transaction::Transaction;
use rust_decimal::Decimal;
use std::collections::VecDeque;
use tokio::join;
use tokio::sync::mpsc;

async fn read(input: &str, expected: impl Into<VecDeque<Transaction>>) {
    let mut expected = expected.into();
    let (tx, mut rx) = mpsc::channel(8);
    let reader = reader_task(input.as_bytes(), tx);

    let test = async move {
        while let Some(actual_transaction) = rx.recv().await {
            match expected.pop_front() {
                Some(expected_transaction) => {
                    assert_eq!(actual_transaction, expected_transaction);
                }
                None => {
                    panic!("Unexpected transaction: {actual_transaction:?}",);
                }
            }
        }
    };
    let (read_result, ()) = join!(reader, test);
    assert!(read_result.is_ok());
}

async fn read_err(input: &str) {
    let (tx, _rx) = mpsc::channel(8);
    reader_task(input.as_bytes(), tx)
        .await
        .expect_err("Expected a read failure");
}

#[tokio::test]
async fn withdrawal() {
    let input = "type, client,  tx,amount
withdrawal, 1,   1,  1.0";

    read(input, vec![Transaction::withdrawal(1, 1, Decimal::from(1))]).await;
}

#[tokio::test]
async fn deposit() {
    let input = "type, client,  tx,amount
deposit, 1,   1,  1.0";

    read(input, vec![Transaction::deposit(1, 1, Decimal::from(1))]).await;
}

#[tokio::test]
async fn dispute() {
    let input = "type, client,  tx,amount
dispute, 1,   1";

    read(input, vec![Transaction::dispute(1, 1)]).await;
}

#[tokio::test]
async fn dispute_amount() {
    let input = "type, client,  tx,amount
dispute, 1,   1, 1";

    read_err(input).await;
}

#[tokio::test]
async fn resolve() {
    let input = "type, client,  tx,amount
resolve, 1,   1";

    read(input, vec![Transaction::resolve(1, 1)]).await;
}

#[tokio::test]
async fn resolve_amount() {
    let input = "type, client,  tx,amount
resolve, 1,   1, 1";

    read_err(input).await;
}

#[tokio::test]
async fn chargeback() {
    let input = "type, client,  tx,amount
chargeback, 1,   1";

    read(input, vec![Transaction::chargeback(1, 1)]).await;
}

#[tokio::test]
async fn chargeback_amount() {
    let input = "type, client,  tx,amount
chargeback, 1,   1, 1";

    read_err(input).await;
}

#[tokio::test]
async fn no_whitespace() {
    let input = "type,client,tx,amount
chargeback,1,1";

    read(input, vec![Transaction::chargeback(1, 1)]).await;
}

#[tokio::test]
async fn whitespace() {
    let input = "type  ,       client     , tx ,   amount
chargeback        ,        1        , 1 ";

    read(input, vec![Transaction::chargeback(1, 1)]).await;
}

#[tokio::test]
async fn integer_amount() {
    let input = "type, client,  tx,amount
deposit, 1,   1,  1";

    read(input, vec![Transaction::deposit(1, 1, Decimal::from(1))]).await;
}

#[tokio::test]
async fn high_precision() {
    let input = "type, client,  tx,amount
deposit, 1,   1,  1.23456789";

    read(
        input,
        vec![Transaction::deposit(
            1,
            1,
            Decimal::try_from(1.23456789).unwrap(),
        )],
    )
    .await;
}

#[tokio::test]
async fn multiple() {
    let input = "type, client,  tx,amount
withdrawal, 1,   1,  1.0
deposit, 1,   1,  1.0
dispute, 1,   1
resolve, 1,   1
chargeback, 1,   1";

    let expected = vec![
        Transaction::withdrawal(1, 1, Decimal::from(1)),
        Transaction::deposit(1, 1, Decimal::from(1)),
        Transaction::dispute(1, 1),
        Transaction::resolve(1, 1),
        Transaction::chargeback(1, 1),
    ];

    read(input, expected).await;
}

#[tokio::test]
async fn unknown() {
    let input = "type, client,  tx,amount
buy, 1,   1";
    read_err(input).await;
}

#[tokio::test]
async fn casing() {
    let input = "type  ,       client     , tx ,   amount
CHARGEBACK        ,        1        , 1 ";
    read_err(input).await;
}

#[tokio::test]
async fn invalid_type() {
    let input = "type,client,tx,amount
chargeback,1.0,1 ";
    read_err(input).await;
}
