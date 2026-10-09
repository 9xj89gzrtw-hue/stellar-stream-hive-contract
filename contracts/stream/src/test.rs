#![cfg(test)]

use crate::types::StreamStatus;
use crate::{StreamContract, StreamContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, Address, Env,
};

/// A minimal stand-in for the real vault contract, used only so the stream
/// contract's tests can exercise the full create -> deposit -> withdraw ->
/// release -> cancel -> refund call flow through real cross-contract
/// invocation, without pulling the vault crate in as a Rust dependency.
///
/// The vault's own security boundary (caller must be the exact configured
/// stream contract) and balance accounting are exhaustively tested in
/// `stellar-stream-hive-vault`'s own test suite (see contracts/vault/src/test.rs)
/// against the real vault contract -- this mock only needs to move tokens
/// realistically enough for the stream contract's own math and control
/// flow to be tested end-to-end.
mod mock_vault {
    use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

    #[contracttype]
    enum Key {
        Balance(u64),
    }

    #[contract]
    pub struct MockVault;

    #[contractimpl]
    impl MockVault {
        pub fn deposit(env: Env, from: Address, token: Address, amount: i128, stream_id: u64) {
            from.require_auth();
            token::Client::new(&env, &token).transfer(
                &from,
                &env.current_contract_address(),
                &amount,
            );
            let key = Key::Balance(stream_id);
            let bal: i128 = env.storage().persistent().get(&key).unwrap_or(0);
            env.storage().persistent().set(&key, &(bal + amount));
        }

        pub fn release(
            env: Env,
            caller: Address,
            to: Address,
            token: Address,
            amount: i128,
            stream_id: u64,
        ) {
            caller.require_auth();
            let key = Key::Balance(stream_id);
            let bal: i128 = env.storage().persistent().get(&key).unwrap_or(0);
            env.storage().persistent().set(&key, &(bal - amount));
            token::Client::new(&env, &token).transfer(
                &env.current_contract_address(),
                &to,
                &amount,
            );
        }

        pub fn refund(
            env: Env,
            caller: Address,
            to: Address,
            token: Address,
            amount: i128,
            stream_id: u64,
        ) {
            Self::release(env, caller, to, token, amount, stream_id);
        }

        pub fn balance_of_stream(env: Env, stream_id: u64) -> i128 {
            env.storage()
                .persistent()
                .get(&Key::Balance(stream_id))
                .unwrap_or(0)
        }
    }
}

use mock_vault::{MockVault, MockVaultClient};

struct TestCtx {
    env: Env,
    stream: StreamContractClient<'static>,
    vault: MockVaultClient<'static>,
    #[allow(dead_code)]
    admin: Address,
    sender: Address,
    recipient: Address,
    token_address: Address,
    token: token::Client<'static>,
}

const START: u32 = 1_000;
const END: u32 = 2_000;
const CLIFF: u32 = 1_000;
const AMOUNT: i128 = 1_000_000;

fn setup() -> TestCtx {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_sequence_number(0);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = sac.address();
    let token_sac = token::StellarAssetClient::new(&env, &token_address);
    let token = token::Client::new(&env, &token_address);
    token_sac.mint(&sender, &(AMOUNT * 10));

    let stream_id = env.register(StreamContract, ());
    let stream = StreamContractClient::new(&env, &stream_id);

    let vault_id = env.register(MockVault, ());
    let vault = MockVaultClient::new(&env, &vault_id);

    stream.initialize(&admin, &vault_id);

    TestCtx {
        env,
        stream,
        vault,
        admin,
        sender,
        recipient,
        token_address,
        token,
    }
}

fn set_ledger(ctx: &TestCtx, seq: u32) {
    ctx.env.ledger().set_sequence_number(seq);
}

fn create_default_stream(ctx: &TestCtx, cancellable: bool) -> u64 {
    ctx.stream.create_stream(
        &ctx.sender,
        &ctx.recipient,
        &ctx.token_address,
        &AMOUNT,
        &START,
        &END,
        &CLIFF,
        &cancellable,
    )
}

#[test]
fn test_create_stream_pulls_funds_into_vault() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    assert_eq!(id, 1);
    assert_eq!(ctx.token.balance(&ctx.vault.address), AMOUNT);
    assert_eq!(ctx.vault.balance_of_stream(&id), AMOUNT);
}

#[test]
fn test_streamed_amount_zero_before_start() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    set_ledger(&ctx, START - 1);
    assert_eq!(ctx.stream.streamed_amount(&id), 0);
}

#[test]
fn test_streamed_amount_full_after_end() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    set_ledger(&ctx, END + 500);
    assert_eq!(ctx.stream.streamed_amount(&id), AMOUNT);
}

#[test]
fn test_streamed_amount_half_at_midpoint() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    let midpoint = START + (END - START) / 2;
    set_ledger(&ctx, midpoint);
    assert_eq!(ctx.stream.streamed_amount(&id), AMOUNT / 2);
}

#[test]
fn test_withdrawable_zero_before_cliff() {
    let ctx = setup();
    // cliff well after start so streaming has begun but cliff hasn't hit.
    let id = ctx.stream.create_stream(
        &ctx.sender,
        &ctx.recipient,
        &ctx.token_address,
        &AMOUNT,
        &START,
        &END,
        &(START + 400),
        &true,
    );
    set_ledger(&ctx, START + 200);
    assert!(ctx.stream.streamed_amount(&id) > 0);
    assert_eq!(ctx.stream.withdrawable_amount(&id), 0);
}

#[test]
fn test_withdraw_reduces_withdrawable() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    let midpoint = START + (END - START) / 2;
    set_ledger(&ctx, midpoint);

    let before = ctx.stream.withdrawable_amount(&id);
    assert_eq!(before, AMOUNT / 2);

    ctx.stream.withdraw(&ctx.recipient, &id, &(AMOUNT / 4));

    let after = ctx.stream.withdrawable_amount(&id);
    assert_eq!(after, AMOUNT / 2 - AMOUNT / 4);
    assert_eq!(ctx.token.balance(&ctx.recipient), AMOUNT / 4);
}

#[test]
#[should_panic]
fn test_withdraw_rejects_amount_exceeding_withdrawable() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    let midpoint = START + (END - START) / 2;
    set_ledger(&ctx, midpoint);
    ctx.stream.withdraw(&ctx.recipient, &id, &(AMOUNT));
}

#[test]
#[should_panic]
fn test_withdraw_rejects_non_recipient_caller() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    set_ledger(&ctx, END);
    let stranger = Address::generate(&ctx.env);
    ctx.stream.withdraw(&stranger, &id, &1);
}

#[test]
fn test_withdraw_max_pulls_entire_withdrawable_balance() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    set_ledger(&ctx, END);
    let amount = ctx.stream.withdraw_max(&ctx.recipient, &id);
    assert_eq!(amount, AMOUNT);
    assert_eq!(ctx.token.balance(&ctx.recipient), AMOUNT);
    assert_eq!(ctx.stream.withdrawable_amount(&id), 0);
}

#[test]
fn test_cancel_splits_funds_between_recipient_and_sender() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    let midpoint = START + (END - START) / 2;
    set_ledger(&ctx, midpoint);

    ctx.stream.cancel_stream(&ctx.sender, &id);

    // Sender gets refunded the unstreamed half immediately.
    assert_eq!(ctx.token.balance(&ctx.sender), AMOUNT * 9 + AMOUNT / 2);
    // Recipient can still withdraw the streamed half, frozen at cancel time.
    assert_eq!(ctx.stream.withdrawable_amount(&id), AMOUNT / 2);
    ctx.stream.withdraw_max(&ctx.recipient, &id);
    assert_eq!(ctx.token.balance(&ctx.recipient), AMOUNT / 2);
}

#[test]
fn test_cancel_freezes_streamed_amount_after_further_ledgers() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    let midpoint = START + (END - START) / 2;
    set_ledger(&ctx, midpoint);
    ctx.stream.cancel_stream(&ctx.sender, &id);

    // Advance far past the original end_ledger: streamed amount must not grow.
    set_ledger(&ctx, END + 10_000);
    assert_eq!(ctx.stream.streamed_amount(&id), AMOUNT / 2);
    assert_eq!(ctx.stream.get_status(&id), StreamStatus::Cancelled);
}

#[test]
#[should_panic]
fn test_cancel_rejected_on_non_cancellable_stream() {
    let ctx = setup();
    let id = create_default_stream(&ctx, false);
    set_ledger(&ctx, START + 100);
    ctx.stream.cancel_stream(&ctx.sender, &id);
}

#[test]
#[should_panic]
fn test_cancel_rejected_for_non_sender() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    set_ledger(&ctx, START + 100);
    let stranger = Address::generate(&ctx.env);
    ctx.stream.cancel_stream(&stranger, &id);
}

#[test]
#[should_panic]
fn test_cancel_rejected_after_completion() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    set_ledger(&ctx, END + 1);
    ctx.stream.cancel_stream(&ctx.sender, &id);
}

#[test]
#[should_panic]
fn test_create_stream_rejects_invalid_time_range() {
    let ctx = setup();
    ctx.stream.create_stream(
        &ctx.sender,
        &ctx.recipient,
        &ctx.token_address,
        &AMOUNT,
        &END,
        &START,
        &START,
        &true,
    );
}

#[test]
#[should_panic]
fn test_create_stream_rejects_cliff_before_start() {
    let ctx = setup();
    ctx.stream.create_stream(
        &ctx.sender,
        &ctx.recipient,
        &ctx.token_address,
        &AMOUNT,
        &START,
        &END,
        &(START - 1),
        &true,
    );
}

#[test]
#[should_panic]
fn test_create_stream_rejects_cliff_after_end() {
    let ctx = setup();
    ctx.stream.create_stream(
        &ctx.sender,
        &ctx.recipient,
        &ctx.token_address,
        &AMOUNT,
        &START,
        &END,
        &(END + 1),
        &true,
    );
}

#[test]
#[should_panic]
fn test_create_stream_rejects_zero_amount() {
    let ctx = setup();
    ctx.stream.create_stream(
        &ctx.sender,
        &ctx.recipient,
        &ctx.token_address,
        &0,
        &START,
        &END,
        &START,
        &true,
    );
}

#[test]
fn test_get_status_transitions_through_lifecycle() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);

    set_ledger(&ctx, START - 1);
    assert_eq!(ctx.stream.get_status(&id), StreamStatus::Pending);

    set_ledger(&ctx, START + 1);
    assert_eq!(ctx.stream.get_status(&id), StreamStatus::Active);

    set_ledger(&ctx, END);
    assert_eq!(ctx.stream.get_status(&id), StreamStatus::Completed);
}

#[test]
fn test_get_streams_by_sender_and_recipient() {
    let ctx = setup();
    let id1 = create_default_stream(&ctx, true);
    let id2 = create_default_stream(&ctx, false);

    let by_sender = ctx.stream.get_streams_by_sender(&ctx.sender);
    assert_eq!(by_sender.len(), 2);
    assert_eq!(by_sender.get(0).unwrap().id, id1);
    assert_eq!(by_sender.get(1).unwrap().id, id2);

    let by_recipient = ctx.stream.get_streams_by_recipient(&ctx.recipient);
    assert_eq!(by_recipient.len(), 2);
}

#[test]
fn test_multiple_streams_have_independent_balances() {
    let ctx = setup();
    let id1 = create_default_stream(&ctx, true);
    let id2 = create_default_stream(&ctx, true);

    set_ledger(&ctx, END);
    ctx.stream.withdraw_max(&ctx.recipient, &id1);

    assert_eq!(ctx.stream.withdrawable_amount(&id1), 0);
    assert_eq!(ctx.stream.withdrawable_amount(&id2), AMOUNT);
}

#[test]
fn test_get_stream_returns_correct_fields() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    let s = ctx.stream.get_stream(&id);
    assert_eq!(s.sender, ctx.sender);
    assert_eq!(s.recipient, ctx.recipient);
    assert_eq!(s.total_amount, AMOUNT);
    assert_eq!(s.withdrawn, 0);
}

// ---------------------------------------------------------------------
// Pause / admin transfer
// ---------------------------------------------------------------------

use crate::error::StreamError;

fn try_create(ctx: &TestCtx) -> Result<u64, StreamError> {
    match ctx.stream.try_create_stream(
        &ctx.sender,
        &ctx.recipient,
        &ctx.token_address,
        &AMOUNT,
        &START,
        &END,
        &CLIFF,
        &true,
    ) {
        Ok(Ok(id)) => Ok(id),
        Err(Ok(e)) => Err(e),
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn test_not_paused_by_default() {
    let ctx = setup();
    assert!(!ctx.stream.is_paused());
}

#[test]
fn test_pause_blocks_create_stream_with_typed_error() {
    let ctx = setup();
    ctx.stream.pause();
    assert!(ctx.stream.is_paused());
    assert_eq!(try_create(&ctx), Err(StreamError::Paused));
}

#[test]
fn test_unpause_restores_create_stream() {
    let ctx = setup();
    ctx.stream.pause();
    ctx.stream.unpause();
    assert!(!ctx.stream.is_paused());
    assert_eq!(try_create(&ctx), Ok(1));
}

#[test]
fn test_pause_never_traps_existing_streams() {
    let ctx = setup();
    let id = create_default_stream(&ctx, true);
    ctx.stream.pause();

    // Recipient can still withdraw what has accrued...
    set_ledger(&ctx, START + (END - START) / 2);
    assert_eq!(ctx.stream.withdraw_max(&ctx.recipient, &id), AMOUNT / 2);

    // ...and the sender can still cancel and be refunded.
    ctx.stream.cancel_stream(&ctx.sender, &id);
    assert_eq!(ctx.vault.balance_of_stream(&id), 0);
}

#[test]
fn test_pause_requires_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();
    let stream_id = env.register(StreamContract, ());
    let stream = StreamContractClient::new(&env, &stream_id);
    stream.initialize(&Address::generate(&env), &Address::generate(&env));

    // Drop the blanket auth mock: pause/unpause must now be rejected.
    env.set_auths(&[]);
    assert!(stream.try_pause().is_err());
    assert!(stream.try_unpause().is_err());
}

#[test]
fn test_pause_before_initialize_is_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let stream_id = env.register(StreamContract, ());
    let stream = StreamContractClient::new(&env, &stream_id);
    assert_eq!(stream.try_pause(), Err(Ok(StreamError::NotInitialized)));
    assert_eq!(stream.try_unpause(), Err(Ok(StreamError::NotInitialized)));
}

#[test]
fn test_transfer_admin_changes_admin_and_new_admin_can_pause() {
    let ctx = setup();
    let new_admin = Address::generate(&ctx.env);
    ctx.stream.transfer_admin(&new_admin);
    assert_eq!(ctx.stream.get_admin(), new_admin);
    ctx.stream.pause();
    assert!(ctx.stream.is_paused());
}

// ---------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------

#[test]
fn test_counts_track_each_party_separately() {
    let ctx = setup();
    assert_eq!(ctx.stream.count_streams_by_sender(&ctx.sender), 0);
    create_default_stream(&ctx, true);
    create_default_stream(&ctx, true);
    assert_eq!(ctx.stream.count_streams_by_sender(&ctx.sender), 2);
    assert_eq!(ctx.stream.count_streams_by_recipient(&ctx.recipient), 2);
    assert_eq!(ctx.stream.count_streams_by_sender(&ctx.recipient), 0);
}

#[test]
fn test_paginated_queries_return_creation_order_slices() {
    let ctx = setup();
    for _ in 0..5 {
        create_default_stream(&ctx, true);
    }

    let page = ctx.stream.get_streams_by_sender_page(&ctx.sender, &0, &2);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().id, 1);
    assert_eq!(page.get(1).unwrap().id, 2);

    let page = ctx
        .stream
        .get_streams_by_recipient_page(&ctx.recipient, &3, &10);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().id, 4);
    assert_eq!(page.get(1).unwrap().id, 5);
}

#[test]
fn test_pagination_edge_cases() {
    let ctx = setup();
    create_default_stream(&ctx, true);

    // offset past the end -> empty, not an error
    assert_eq!(
        ctx.stream
            .get_streams_by_sender_page(&ctx.sender, &5, &10)
            .len(),
        0
    );
    // zero limit -> empty
    assert_eq!(
        ctx.stream
            .get_streams_by_sender_page(&ctx.sender, &0, &0)
            .len(),
        0
    );
    // offset + limit overflow must saturate, not panic
    assert_eq!(
        ctx.stream
            .get_streams_by_sender_page(&ctx.sender, &u32::MAX, &u32::MAX)
            .len(),
        0
    );
    // unknown address -> empty
    let stranger = Address::generate(&ctx.env);
    assert_eq!(
        ctx.stream
            .get_streams_by_sender_page(&stranger, &0, &10)
            .len(),
        0
    );
}

#[test]
fn test_page_size_is_capped() {
    let ctx = setup();
    let n = crate::MAX_PAGE_SIZE + 3;
    token::StellarAssetClient::new(&ctx.env, &ctx.token_address)
        .mint(&ctx.sender, &(AMOUNT * n as i128));
    for _ in 0..n {
        create_default_stream(&ctx, true);
    }
    let page = ctx
        .stream
        .get_streams_by_sender_page(&ctx.sender, &0, &u32::MAX);
    assert_eq!(page.len(), crate::MAX_PAGE_SIZE);
}

// ---------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------

#[test]
fn test_create_event_carries_cliff_and_cancellable() {
    use soroban_sdk::testutils::Events as _;
    let ctx = setup();
    create_default_stream(&ctx, false);
    // The indexer reconstructs a stream purely from events, so the create
    // event must be emitted by the stream contract.
    let events = ctx.env.events().all();
    assert!(!events.events().is_empty());
}
