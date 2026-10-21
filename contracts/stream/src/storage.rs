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

use soroban_sdk::{contracttype, vec, Address, Env, Vec};

use crate::types::Stream;

/// Extend an entry's TTL once it is at or below this many ledgers
/// (~7 days at 5s ledgers).
const LEDGER_THRESHOLD: u32 = 120_960;
/// Extend an entry's TTL to this many ledgers (~14 days at 5s ledgers).
const LEDGER_BUMP: u32 = 241_920;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Vault,
    NextStreamId,
    Paused,
    Stream(u64),
    SenderStreams(Address),
    RecipientStreams(Address),
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

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn set_vault(env: &Env, vault: &Address) {
    env.storage().instance().set(&DataKey::Vault, vault);
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn get_vault(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Vault)
}

/// Allocates and returns the next stream id (starting at 1).
pub fn next_stream_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextStreamId)
        .unwrap_or(0);
    let next = current + 1;
    env.storage().instance().set(&DataKey::NextStreamId, &next);
    next
}

pub fn get_stream(env: &Env, stream_id: u64) -> Option<Stream> {
    env.storage().persistent().get(&DataKey::Stream(stream_id))
}

pub fn set_stream(env: &Env, stream: &Stream) {
    let key = DataKey::Stream(stream.id);
    env.storage().persistent().set(&key, stream);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn get_sender_streams(env: &Env, sender: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::SenderStreams(sender.clone()))
        .unwrap_or_else(|| vec![env])
}

pub fn add_sender_stream(env: &Env, sender: &Address, stream_id: u64) {
    let mut list = get_sender_streams(env, sender);
    list.push_back(stream_id);
    let key = DataKey::SenderStreams(sender.clone());
    env.storage().persistent().set(&key, &list);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn get_recipient_streams(env: &Env, recipient: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::RecipientStreams(recipient.clone()))
        .unwrap_or_else(|| vec![env])
}

pub fn add_recipient_stream(env: &Env, recipient: &Address, stream_id: u64) {
    let mut list = get_recipient_streams(env, recipient);
    list.push_back(stream_id);
    let key = DataKey::RecipientStreams(recipient.clone());
    env.storage().persistent().set(&key, &list);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}
