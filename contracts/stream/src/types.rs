use soroban_sdk::{contracttype, Address};

/// Lifecycle state of a stream. `Pending`/`Active`/`Completed` are always
/// derived from the current ledger sequence versus `start_ledger` /
/// `end_ledger` — they are never persisted as-is. `Cancelled` is the one
/// terminal state that a sender can force early, and it IS persisted
/// (recorded in `Stream.status`) because it can't be derived from time
/// alone.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamStatus {
    /// Created, but `start_ledger` has not been reached yet.
    Pending,
    /// Between `start_ledger` and `end_ledger`; actively accruing.
    Active,
    /// `end_ledger` has been reached or passed; fully streamed.
    Completed,
    /// Cancelled early by the sender before completion.
    Cancelled,
}

/// A single payment stream.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    /// Unique id assigned by `create_stream`.
    pub id: u64,
    /// Account that funded the stream and may cancel it when `cancellable`.
    pub sender: Address,
    /// Account that receives the streamed funds and may withdraw them.
    pub recipient: Address,
    /// Token contract the stream pays in.
    pub token: Address,
    /// Total amount deposited into the stream, in the token's base units.
    pub total_amount: i128,
    /// Amount the recipient has withdrawn so far, in the token's base units.
    pub withdrawn: i128,
    /// Ledger sequence at which accrual begins.
    pub start_ledger: u32,
    /// Ledger sequence at which the stream has fully accrued `total_amount`.
    pub end_ledger: u32,
    /// Ledger sequence before which nothing is withdrawable. Equal to
    /// `start_ledger` when the stream has no cliff.
    pub cliff_ledger: u32,
    /// Whether the sender may cancel the stream.
    pub cancellable: bool,
    /// Persisted lifecycle state. Only `Cancelled` is stored; the other
    /// states are derived from the current ledger.
    pub status: StreamStatus,
    /// Ledger sequence at which the stream was created. This is a ledger
    /// number, not a timestamp.
    pub created_at: u32,
    /// Ledger sequence at which the stream was cancelled. `0` if it has
    /// never been cancelled. Used to freeze `streamed_amount` at the exact
    /// value it held the moment of cancellation, so ledger progress after
    /// cancellation can never inflate what the recipient is owed.
    pub cancelled_at_ledger: u32,
}
