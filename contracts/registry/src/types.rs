use soroban_sdk::{contracttype, Address};

/// Mirrors `StreamStatus` from the stream contract. Kept as an independent
/// definition (rather than a crate dependency) so the registry stays fully
/// decoupled and can be deployed, upgraded, or replaced without touching
/// the stream contract's build graph.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

/// What the registry knows about one stream. This is a summary record for
/// indexing/analytics purposes — the stream contract remains the source of
/// truth for a stream's authoritative state.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamRecord {
    /// Id of the stream in the stream contract.
    pub stream_id: u64,
    /// Account that funded the stream.
    pub sender: Address,
    /// Account that receives the streamed funds.
    pub recipient: Address,
    /// Token contract the stream pays in.
    pub token: Address,
    /// Total amount registered for the stream (its `total_amount`). Set once
    /// by `register_stream` and never changed afterwards.
    pub amount: i128,
    /// Absolute withdrawn amount as last reported by `update_stream`. This is
    /// not a delta, so each update replaces the previous value.
    pub withdrawn: i128,
    /// Status as last reported by `register_stream` or `update_stream`. It
    /// may lag the stream contract until the next update.
    pub status: StreamStatus,
}

/// Aggregate, protocol-wide statistics maintained incrementally as streams
/// are registered and updated.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryStats {
    pub total_streams: u64,
    pub active_streams: u64,
    /// Cumulative `amount` committed across every stream ever registered
    /// (a lifetime "total value streamed" metric, not a live balance).
    pub total_streamed: i128,
    /// Cumulative amount withdrawn across all streams, as last reported by
    /// `update_stream`.
    pub total_withdrawn: i128,
}
