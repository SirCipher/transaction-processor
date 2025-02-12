mod client;
mod read;
mod store;
mod transaction;

use crate::client::AccountPrinter;
use crate::read::reader_task;
use crate::{
    client::Client,
    store::{AccountStore, TransactionStore},
    transaction::Transaction,
};
use futures::Stream;
use futures_util::StreamExt;
use lru::LruCache;
use std::env;
use std::fs::File;
use std::num::NonZeroUsize;
use tokio::join;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Maximum number of accounts to hold in the LRU cache.
const MAX_ACCOUNTS: usize = 2048;
/// MPSC bridge channel capacity.
const BRIDGE_CAPACITY: usize = 128;

#[tokio::main]
async fn main() {
    let mut args = env::args().skip(1);
    let command = args.next();

    match command.as_deref() {
        Some(path) => {
            let file = File::open(path).expect("File not found");
            let (bridge_tx, bridge_rx) = mpsc::channel(BRIDGE_CAPACITY);

            let account_store = AccountStore::default();
            let producer_task = reader_task(file, bridge_tx);
            let consumer_task =
                consumer_task(account_store.clone(), ReceiverStream::new(bridge_rx));

            let (producer_result, ()) = join!(producer_task, consumer_task);
            producer_result.expect("CSV reader error");

            print_accounts(account_store);
        }
        None => println!("Missing CSV file path argument"),
    }
}

/// A task which will consume a stream of transactions from `stream` and execute them against
/// clients.
///
/// The task maintains `MAX_ACCOUNTS` tasks in an LRU cache to reduce the number of channels active
/// and when an entry is evicted pending transactions are processed before it is terminated.
pub async fn consumer_task<S>(account_store: AccountStore, mut stream: S)
where
    S: Stream<Item = Transaction> + Unpin,
{
    let mut clients: LruCache<u16, Client> =
        LruCache::new(NonZeroUsize::new(MAX_ACCOUNTS).expect("Invalid max accounts size"));
    let transaction_store = TransactionStore::default();

    while let Some(transaction) = stream.next().await {
        let client_id = transaction.client_id;
        match clients.get(&client_id) {
            Some(client) => client.execute_transaction(transaction).await,
            None => {
                let client =
                    Client::from_store(account_store.clone(), transaction_store.clone(), client_id);
                client.execute_transaction(transaction).await;

                if let Some((_, old_client)) = clients.push(client_id, client) {
                    old_client.stop().await;
                }
            }
        }
    }

    for (_, client) in clients.into_iter() {
        client.stop().await;
    }
}

/// Prints the state of every account to the standard output stream.
fn print_accounts(store: AccountStore) {
    println!("client,\tavailable,\theld,\ttotal,\tlocked");

    store.for_each(|account| {
        println!("{}", AccountPrinter::new(account));
    });
}
