//! Typed contract events, defined with `#[contractevent]` rather than
//! published as untyped tuples, so they show up in the contract's
//! interface specification.
use soroban_sdk::{contractevent, Address};

#[contractevent(data_format = "vec")]
pub struct Create {
    #[topic]
    pub stream_id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub token: Address,
    pub total_amount: i128,
    pub start_ledger: u32,
    pub end_ledger: u32,
    pub cliff_ledger: u32,
    pub cancellable: bool,
}

#[contractevent(data_format = "vec")]
pub struct Withdraw {
    #[topic]
    pub stream_id: u64,
    pub recipient: Address,
    pub amount: i128,
}

#[contractevent(data_format = "vec")]
pub struct Cancel {
    #[topic]
    pub stream_id: u64,
    pub sender: Address,
    pub streamed_now: i128,
    pub remainder: i128,
}

/// Emitted when the admin pauses new stream creation.
#[contractevent(data_format = "vec")]
pub struct Pause {
    pub admin: Address,
}

/// Emitted when the admin resumes new stream creation.
#[contractevent(data_format = "vec")]
pub struct Unpause {
    pub admin: Address,
}

/// Emitted when the admin role is handed to a new address.
#[contractevent(data_format = "vec")]
pub struct AdminTransfer {
    pub previous_admin: Address,
    pub new_admin: Address,
}
