use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Errors which may be produced when performing operations on the Balance.
#[derive(Debug, Copy, Clone, PartialEq, thiserror::Error)]
pub enum InvalidOperation {
    /// An attempt was made to perform a negative deposit.
    #[error("Negative deposit")]
    NegativeDeposit,
    /// An attempt was made to perform a negative withdrawal.
    #[error("Negative withdrawal")]
    NegativeWithdrawal,
    /// The value of a withdrawal was too large and an underflow occurred.
    #[error("Operation underflow")]
    Underflow,
    /// The value of a deposit was too large and an overflow occurred.
    #[error("Operation overflow")]
    Overflow,
    /// Insufficient funds are available to perform the requested withdrawal.
    #[error("Insufficient funds")]
    InsufficientFunds,
}

/// A account's balance model.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Balance {
    /// The amount available. New deposits increase this amount and withdrawals decrease it.
    available: Decimal,
    /// The amount held due to disputes.
    held: Decimal,
}

impl Balance {
    /// Returns the amount available for withdrawal.
    pub fn available(&self) -> Decimal {
        self.available
    }

    /// Returns the amount held due to disputes.
    pub fn held(&self) -> Decimal {
        self.held
    }

    /// Returns the sum of the available and held funds.
    pub fn total(&self) -> Decimal {
        self.available + self.held
    }

    /// Decreases the available funds by `amount` and increases the held funds by `amount`. This
    /// may cause the available funds to go into a negative amount.
    pub fn hold(&mut self, amount: Decimal) {
        self.available -= amount;
        self.held += amount;
    }

    /// Releases `amount` from the held funds and increases the available funds by `amount`.
    pub fn release(&mut self, amount: Decimal) {
        self.available += amount;
        self.held -= amount;
    }

    /// Decreases the held funds by `amount`.
    pub fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
    }

    /// Attempts to deposit `amount`.
    ///
    /// # Errors
    /// Returns an error if `amount` is negative or an overflow would occur.
    pub fn deposit(&mut self, amount: Decimal) -> Result<(), InvalidOperation> {
        if amount.is_sign_negative() {
            Err(InvalidOperation::NegativeDeposit)
        } else {
            match self.available.checked_add(amount) {
                Some(amount) => {
                    self.available = amount;
                    Ok(())
                }
                None => Err(InvalidOperation::Overflow),
            }
        }
    }

    /// Attempts to withdraw `amount`.
    ///
    /// # Errors
    /// Returns an error if `amount` is negative or an underflow would occur.
    pub fn withdraw(&mut self, amount: Decimal) -> Result<(), InvalidOperation> {
        if amount.is_sign_negative() {
            Err(InvalidOperation::NegativeWithdrawal)
        } else {
            match self.available.checked_sub(amount) {
                Some(amount) => {
                    if amount.is_sign_negative() {
                        Err(InvalidOperation::InsufficientFunds)
                    } else {
                        self.available = amount;
                        Ok(())
                    }
                }
                None => Err(InvalidOperation::Underflow),
            }
        }
    }
}
