//! Minimal cross-contract call helpers for talking to the vault contract.
//!
//! Rather than depending on the vault crate directly (which would force a
//! fixed build order and a binary WASM artifact dependency between crates),
//! the stream contract calls the vault purely by address + function name
//! using [`Env::invoke_contract`]. This keeps the two contracts fully
//! decoupled at compile time while still being exactly how a generated
//! `Client` would call it under the hood.
//!
//! The argument vectors below are built positionally, so the order and types
//! must match the vault's signatures exactly. A mismatch is not caught at
//! compile time and only surfaces as a runtime failure. The stream tests use a
//! mock vault with the same signatures (see `mock_vault` in `test.rs`).
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol};

/// Mirrors the vault's `deposit`:
/// `deposit(env, from: Address, token: Address, amount: i128, stream_id: u64)`.
pub fn deposit(
    env: &Env,
    vault: &Address,
    from: &Address,
    token: &Address,
    amount: i128,
    stream_id: u64,
) {
    let args = vec![
        env,
        from.into_val(env),
        token.into_val(env),
        amount.into_val(env),
        stream_id.into_val(env),
    ];
    let _: () = env.invoke_contract(vault, &Symbol::new(env, "deposit"), args);
}

/// Mirrors the vault's `release`:
/// `release(env, caller: Address, to: Address, token: Address, amount: i128, stream_id: u64)`.
pub fn release(
    env: &Env,
    vault: &Address,
    caller: &Address,
    to: &Address,
    token: &Address,
    amount: i128,
    stream_id: u64,
) {
    let args = vec![
        env,
        caller.into_val(env),
        to.into_val(env),
        token.into_val(env),
        amount.into_val(env),
        stream_id.into_val(env),
    ];
    let _: () = env.invoke_contract(vault, &Symbol::new(env, "release"), args);
}

/// Mirrors the vault's `refund`:
/// `refund(env, caller: Address, to: Address, token: Address, amount: i128, stream_id: u64)`.
pub fn refund(
    env: &Env,
    vault: &Address,
    caller: &Address,
    to: &Address,
    token: &Address,
    amount: i128,
    stream_id: u64,
) {
    let args = vec![
        env,
        caller.into_val(env),
        to.into_val(env),
        token.into_val(env),
        amount.into_val(env),
        stream_id.into_val(env),
    ];
    let _: () = env.invoke_contract(vault, &Symbol::new(env, "refund"), args);
}
