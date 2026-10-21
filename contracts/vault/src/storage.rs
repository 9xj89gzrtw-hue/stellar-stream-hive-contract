//! Storage layout for this contract.
//!
//! Soroban has three storage types. This module uses two:
//!
//! - **Instance** storage is a single ledger entry shared by the whole
//!   contract. It holds small, contract-wide values such as the admin
//!   address and counters. All instance keys share one TTL, so extending it
//!   keeps every instance key alive.
//! - **Persistent** storage has one ledger entry per key. It holds per-record
//!   data (streams, balances, records, index lists). Each entry has its own
//!   TTL, so an entry expires independently of the others.
//!
//! TTL is measured in ledgers. A write extends an entry's TTL once it is at
//! or below `LEDGER_THRESHOLD`, and extends it to `LEDGER_BUMP`. Reads do not
//! extend the TTL, so an entry that is only read still expires on schedule.
//! The write helpers below extend the TTL of the entry they touch.

use soroban_sdk::{contracttype, Address, Env};

/// Extend an entry's TTL once it is at or below this many ledgers
/// (~7 days at 5s ledgers).
const LEDGER_THRESHOLD: u32 = 120_960;
/// Extend an entry's TTL to this many ledgers (~14 days at 5s ledgers).
const LEDGER_BUMP: u32 = 241_920;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    StreamContract,
    /// Locked balance currently attributed to a given stream id.
    StreamBalance(u64),
    /// The token a given stream id was funded with (set on first deposit).
    StreamToken(u64),
    /// Running total of a token's balance locked in the vault across all
    /// streams.
    TotalLocked(Address),
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_stream_contract(env: &Env, addr: &Address) {
    env.storage().instance().set(&DataKey::StreamContract, addr);
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn get_stream_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::StreamContract)
}

pub fn get_stream_balance(env: &Env, stream_id: u64) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::StreamBalance(stream_id))
        .unwrap_or(0)
}

pub fn set_stream_balance(env: &Env, stream_id: u64, amount: i128) {
    let key = DataKey::StreamBalance(stream_id);
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn get_stream_token(env: &Env, stream_id: u64) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::StreamToken(stream_id))
}

pub fn set_stream_token(env: &Env, stream_id: u64, token: &Address) {
    let key = DataKey::StreamToken(stream_id);
    env.storage().persistent().set(&key, token);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn get_total_locked(env: &Env, token: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalLocked(token.clone()))
        .unwrap_or(0)
}

pub fn set_total_locked(env: &Env, token: &Address, amount: i128) {
    let key = DataKey::TotalLocked(token.clone());
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}
