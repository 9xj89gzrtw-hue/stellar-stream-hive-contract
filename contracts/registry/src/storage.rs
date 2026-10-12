use soroban_sdk::{contracttype, vec, Env, Vec};

use crate::types::{RegistryStats, StreamRecord};

const LEDGER_THRESHOLD: u32 = 120_960; // ~7 days at 5s ledgers
const LEDGER_BUMP: u32 = 241_920; // ~14 days at 5s ledgers

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    /// Number of registered streams (instance storage).
    StreamCount,
    /// The id registered at a given registration index (persistent storage).
    StreamIdAt(u32),
    Record(u64),
    Stats,
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &soroban_sdk::Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn get_admin(env: &Env) -> Option<soroban_sdk::Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn get_stream_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::StreamCount)
        .unwrap_or(0)
}

/// Ids are stored one ledger entry per registration index rather than in a
/// single growing `Vec`, so registering a stream is O(1) and can never hit
/// the ledger-entry size limit no matter how many streams exist.
pub fn add_stream_id(env: &Env, stream_id: u64) {
    let index = get_stream_count(env);
    let key = DataKey::StreamIdAt(index);
    env.storage().persistent().set(&key, &stream_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);

    env.storage()
        .instance()
        .set(&DataKey::StreamCount, &(index + 1));
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

/// Ids in registration order, starting at `offset`, at most `limit` of them.
pub fn get_stream_ids_page(env: &Env, offset: u32, limit: u32) -> Vec<u64> {
    let count = get_stream_count(env);
    let end = offset.saturating_add(limit).min(count);
    let mut out = vec![env];
    let mut index = offset;
    while index < end {
        if let Some(id) = env.storage().persistent().get(&DataKey::StreamIdAt(index)) {
            out.push_back(id);
        }
        index += 1;
    }
    out
}

pub fn get_all_stream_ids(env: &Env) -> Vec<u64> {
    get_stream_ids_page(env, 0, get_stream_count(env))
}

pub fn get_record(env: &Env, stream_id: u64) -> Option<StreamRecord> {
    env.storage().persistent().get(&DataKey::Record(stream_id))
}

pub fn set_record(env: &Env, record: &StreamRecord) {
    let key = DataKey::Record(record.stream_id);
    env.storage().persistent().set(&key, record);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn get_stats(env: &Env) -> RegistryStats {
    env.storage()
        .instance()
        .get(&DataKey::Stats)
        .unwrap_or(RegistryStats {
            total_streams: 0,
            active_streams: 0,
            total_streamed: 0,
            total_withdrawn: 0,
        })
}

pub fn set_stats(env: &Env, stats: &RegistryStats) {
    env.storage().instance().set(&DataKey::Stats, stats);
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}
