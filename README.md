# Rust toy payments engine

This repository contains a simple accounting transaction machine implemented in Rust. This is an improved version over
the last implementation which is available on GitHub [here](https://github.com/SirCipher/transaction-machine-1).

To run the application:

```
cargo run input.csv
```

Where `input.csv` is the name of the CSV file to process.

# Assumptions

- Only a deposit can be disputed.
- It is possible for a client's balance to drop below zero if a deposit that was made has been withdrawn and then a
  dispute is made on the original transaction.
- Negative transaction amounts cannot be processed.
- Since the use of RocksDB is not allowed, the state of all accounts and deposit transactions are held in memory.
- The CLI argument is an absolute path to the CSV file to process.

# Improvements vs the last implementation

Since this application has been built before there are some improvements vs the original implementation:

- A client's balance is represented using a `Decimal` type instead of an `f64` to avoid rounding errors.
- The removal of a client's processing task is handled more gracefully and all pending transactions are flushed before
  the task terminates.
- The transaction execution logic has been simplified.
- CSV record deserialization is has been simplified.
- Persisting a type is now infallible and so error handling is now encapsulated within the client; previously, the
  application would terminate on a store error. As errors cannot be printed to the standard output stream, they are
  sunk.

# Design

As there is a guarantee that transactions for a given client increase monotonically in the CSV file, clients have a
mailbox and transactions are pushed into a given clients mailbox so they can be processed sequentially. Clients run as
async tasks to increase the transaction throughput of the application.
