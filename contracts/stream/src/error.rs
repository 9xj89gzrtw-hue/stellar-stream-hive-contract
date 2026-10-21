use soroban_sdk::contracterror;

/// All error conditions the stream contract can return.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StreamError {
    /// `initialize` was called a second time.
    AlreadyInitialized = 1,
    /// A function that requires `initialize` to have run first was called
    /// before it was.
    NotInitialized = 2,
    /// The caller authenticated successfully but is not the party allowed
    /// to perform this action (e.g. not the stream's recipient/sender).
    Unauthorized = 3,
    /// An amount argument was zero or negative.
    InvalidAmount = 4,
    /// `end_ledger` was not strictly after `start_ledger`.
    InvalidTimeRange = 5,
    /// `cliff_ledger` was outside `[start_ledger, end_ledger]`.
    InvalidCliff = 6,
    /// No stream exists with the given id.
    StreamNotFound = 7,
    /// Reserved and currently unused: no code path returns it. A withdrawal
    /// before `cliff_ledger` fails with `ExceedsWithdrawable`, because
    /// `withdrawable_amount` is 0 until the cliff. Kept so the error codes
    /// stay stable; do not renumber or remove it.
    BeforeCliff = 8,
    /// A withdrawal amount exceeds what is currently withdrawable.
    ExceedsWithdrawable = 9,
    /// `cancel_stream` was called on a stream created with
    /// `cancellable = false`.
    NotCancellable = 10,
    /// `cancel_stream` was called on a stream that is already cancelled or
    /// has already run to completion.
    AlreadyFinalized = 11,
    /// Checked arithmetic overflowed or underflowed.
    MathOverflow = 12,
    /// `create_stream` was called while the contract is paused. Withdrawals
    /// and cancellations are never blocked by a pause, so funds already in
    /// streams can always be reached.
    Paused = 13,
}
