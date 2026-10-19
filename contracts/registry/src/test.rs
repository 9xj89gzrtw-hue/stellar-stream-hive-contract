#![cfg(test)]

use crate::{RegistryContract, RegistryContractClient, RegistryError, StreamStatus};
use soroban_sdk::{testutils::Address as _, Address, Env};

struct TestCtx {
    env: Env,
    registry: RegistryContractClient<'static>,
    admin: Address,
}

fn setup() -> TestCtx {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register(RegistryContract, ());
    let registry = RegistryContractClient::new(&env, &id);
    registry.initialize(&admin);
    TestCtx {
        env,
        registry,
        admin,
    }
}

#[test]
fn test_initialize_sets_empty_stats() {
    let ctx = setup();
    let stats = ctx.registry.get_stats();
    assert_eq!(stats.total_streams, 0);
    assert_eq!(stats.active_streams, 0);
    assert_eq!(stats.total_streamed, 0);
    assert_eq!(stats.total_withdrawn, 0);
}

#[test]
#[should_panic]
fn test_initialize_twice_rejected() {
    let ctx = setup();
    ctx.registry.initialize(&ctx.admin);
}

#[test]
fn test_get_active_streams_empty_after_initialize() {
    let ctx = setup();
    let active = ctx.registry.get_active_streams();
    assert_eq!(active, soroban_sdk::Vec::<u64>::new(&ctx.env));
    assert_eq!(active.len(), 0);
    assert_eq!(ctx.registry.stream_count(), 0);
}

#[test]
fn test_register_stream_adds_to_all_streams() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);

    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);

    let all = ctx.registry.get_all_streams();
    assert_eq!(all.len(), 1);
    assert_eq!(all.get(0).unwrap(), 1);
}

#[test]
fn test_register_stream_updates_stats() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);

    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);
    ctx.registry
        .register_stream(&2, &sender, &recipient, &token, &500);

    let stats = ctx.registry.get_stats();
    assert_eq!(stats.total_streams, 2);
    assert_eq!(stats.active_streams, 2);
    assert_eq!(stats.total_streamed, 1500);
}

#[test]
#[should_panic]
fn test_register_stream_rejects_duplicate_id() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);

    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &500);
}

#[test]
fn test_update_stream_tracks_withdrawn_delta() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);

    ctx.registry.update_stream(&1, &300, &StreamStatus::Active);
    assert_eq!(ctx.registry.get_stats().total_withdrawn, 300);

    ctx.registry.update_stream(&1, &700, &StreamStatus::Active);
    assert_eq!(ctx.registry.get_stats().total_withdrawn, 700);
}

#[test]
fn test_lowering_withdrawn_does_not_reduce_total_withdrawn() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);

    ctx.registry.update_stream(&1, &100, &StreamStatus::Active);
    ctx.registry.update_stream(&1, &40, &StreamStatus::Active);

    assert_eq!(ctx.registry.get_stream_record(&1).withdrawn, 40);
    assert_eq!(ctx.registry.get_stats().total_withdrawn, 100);
}

#[test]
fn test_get_admin_before_initialize_returns_not_initialized() {
    let env = Env::default();
    let id = env.register(RegistryContract, ());
    let registry = RegistryContractClient::new(&env, &id);

    assert_eq!(registry.try_get_admin(), Err(Ok(RegistryError::NotInitialized)));
}

#[test]
fn test_update_stream_to_pending_decrements_active_count() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);
    assert_eq!(ctx.registry.get_stats().active_streams, 1);

    ctx.registry.update_stream(&1, &0, &StreamStatus::Pending);
    assert_eq!(ctx.registry.get_stats().active_streams, 0);

    ctx.registry.update_stream(&1, &0, &StreamStatus::Active);
    assert_eq!(ctx.registry.get_stats().active_streams, 1);
}

#[test]
fn test_cancelled_to_completed_keeps_active_count_at_zero() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);

    ctx.registry
        .update_stream(&1, &0, &StreamStatus::Cancelled);
    assert_eq!(ctx.registry.get_stats().active_streams, 0);

    ctx.registry
        .update_stream(&1, &0, &StreamStatus::Completed);
    assert_eq!(ctx.registry.get_stats().active_streams, 0);
}

#[test]
fn test_update_stream_to_completed_decrements_active_count() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);
    assert_eq!(ctx.registry.get_stats().active_streams, 1);

    ctx.registry
        .update_stream(&1, &1000, &StreamStatus::Completed);
    assert_eq!(ctx.registry.get_stats().active_streams, 0);
}

#[test]
fn test_update_stream_to_cancelled_decrements_active_count() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);

    ctx.registry
        .update_stream(&1, &400, &StreamStatus::Cancelled);
    let stats = ctx.registry.get_stats();
    assert_eq!(stats.active_streams, 0);
    assert_eq!(stats.total_withdrawn, 400);
}

#[test]
#[should_panic]
fn test_update_stream_rejects_unknown_id() {
    let ctx = setup();
    ctx.registry
        .update_stream(&999, &100, &StreamStatus::Active);
}

#[test]
fn test_get_active_streams_filters_correctly() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);
    ctx.registry
        .register_stream(&2, &sender, &recipient, &token, &2000);
    ctx.registry
        .register_stream(&3, &sender, &recipient, &token, &3000);

    ctx.registry
        .update_stream(&2, &2000, &StreamStatus::Completed);

    let active = ctx.registry.get_active_streams();
    assert_eq!(active.len(), 2);
    assert_eq!(active.get(0).unwrap(), 1);
    assert_eq!(active.get(1).unwrap(), 3);
}

#[test]
fn test_get_stream_record_returns_current_values() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);
    ctx.registry.update_stream(&1, &250, &StreamStatus::Active);

    let record = ctx.registry.get_stream_record(&1);
    assert_eq!(record.stream_id, 1);
    assert_eq!(record.amount, 1000);
    assert_eq!(record.withdrawn, 250);
    assert_eq!(record.status, StreamStatus::Active);
}

#[test]
#[should_panic]
fn test_get_stream_record_rejects_unknown_id() {
    let ctx = setup();
    ctx.registry.get_stream_record(&42);
}

#[test]
fn test_reactivating_a_stream_increments_active_count() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);
    ctx.registry.update_stream(&1, &0, &StreamStatus::Pending);
    assert_eq!(ctx.registry.get_stats().active_streams, 0);

    ctx.registry.update_stream(&1, &0, &StreamStatus::Active);
    assert_eq!(ctx.registry.get_stats().active_streams, 1);
}

#[test]
fn test_stream_count_tracks_registrations() {
    let ctx = setup();
    assert_eq!(ctx.registry.stream_count(), 0);
    register_n(&ctx, 3);
    assert_eq!(ctx.registry.stream_count(), 3);
}

#[test]
fn test_stream_count_matches_stats_total_streams() {
    let ctx = setup();
    assert_eq!(
        ctx.registry.stream_count(),
        ctx.registry.get_stats().total_streams
    );

    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    for i in 0..4u64 {
        ctx.registry
            .register_stream(&(200 + i), &sender, &recipient, &token, &1000);
        assert_eq!(
            ctx.registry.stream_count(),
            ctx.registry.get_stats().total_streams
        );
    }

    // a rejected duplicate must not let the two counters diverge
    assert!(ctx
        .registry
        .try_register_stream(&200, &sender, &recipient, &token, &1)
        .is_err());
    assert_eq!(
        ctx.registry.stream_count(),
        ctx.registry.get_stats().total_streams
    );
    assert_eq!(ctx.registry.stream_count(), 4);
}

#[test]
fn test_register_stream_emits_event() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);

    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);

    let events = ctx.env.events().all();
    let event = events.last().unwrap();
    assert_eq!(
        event.0,
        soroban_sdk::vec![&ctx.env, ctx.registry.address.clone()]
    );
    assert_eq!(
        event.1,
        soroban_sdk::vec![
            &ctx.env,
            soroban_sdk::Symbol::new(&ctx.env, "register"),
            soroban_sdk::Symbol::new(&ctx.env, "stream"),
        ]
    );
    let data: (u64, i128) = soroban_sdk::FromVal::from_val(&ctx.env, &event.2);
    assert_eq!(data, (1, 1000));
}

#[test]
fn test_update_stream_emits_event() {
    let ctx = setup();
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    ctx.registry
        .register_stream(&1, &sender, &recipient, &token, &1000);

    ctx.registry.update_stream(&1, &300, &StreamStatus::Active);

    let events = ctx.env.events().all();
    let event = events.last().unwrap();
    assert_eq!(
        event.0,
        soroban_sdk::vec![&ctx.env, ctx.registry.address.clone()]
    );
    assert_eq!(
        event.1,
        soroban_sdk::vec![
            &ctx.env,
            soroban_sdk::Symbol::new(&ctx.env, "update"),
            soroban_sdk::Symbol::new(&ctx.env, "stream"),
        ]
    );
    let data: (u64, i128) = soroban_sdk::FromVal::from_val(&ctx.env, &event.2);
    assert_eq!(data, (1, 300));
}

#[test]
fn test_get_streams_page_zero_limit_is_empty() {
    let ctx = setup();
    register_n(&ctx, 3);
    let page = ctx.registry.get_streams_page(&0, &0);
    assert_eq!(page, soroban_sdk::Vec::<u64>::new(&ctx.env));
    assert_eq!(page.len(), 0);
}

#[test]
fn test_get_streams_page_returns_registration_order_slices() {
    let ctx = setup();
    register_n(&ctx, 10);

    let first = ctx.registry.get_streams_page(&0, &4);
    assert_eq!(first.len(), 4);
    assert_eq!(first.get(0).unwrap(), 100);
    assert_eq!(first.get(3).unwrap(), 103);

    // a window that overlaps the end is truncated, not padded
    let tail = ctx.registry.get_streams_page(&8, &4);
    assert_eq!(tail.len(), 2);
    assert_eq!(tail.get(0).unwrap(), 108);
    assert_eq!(tail.get(1).unwrap(), 109);
}

#[test]
fn test_get_streams_page_past_the_end_is_empty() {
    let ctx = setup();
    register_n(&ctx, 3);
    assert_eq!(ctx.registry.get_streams_page(&3, &10).len(), 0);
    assert_eq!(ctx.registry.get_streams_page(&u32::MAX, &u32::MAX).len(), 0);
    assert_eq!(ctx.registry.get_streams_page(&0, &0).len(), 0);
}

#[test]
fn test_get_streams_page_caps_limit_at_max_page_size() {
    let ctx = setup();
    register_n(&ctx, 60);
    let page = ctx.registry.get_streams_page(&0, &1_000);
    assert_eq!(page.len(), crate::MAX_PAGE_SIZE);
    // the next page picks up exactly where the capped one stopped
    let next = ctx.registry.get_streams_page(&crate::MAX_PAGE_SIZE, &1_000);
    assert_eq!(next.len(), 10);
    assert_eq!(next.get(0).unwrap(), 100 + u64::from(crate::MAX_PAGE_SIZE));
}

#[test]
fn test_paging_through_everything_matches_get_all_streams() {
    let ctx = setup();
    register_n(&ctx, 120);

    let mut paged = soroban_sdk::Vec::<u64>::new(&ctx.env);
    let mut offset = 0;
    loop {
        let page = ctx
            .registry
            .get_streams_page(&offset, &crate::MAX_PAGE_SIZE);
        if page.is_empty() {
            break;
        }
        offset += page.len();
        paged.append(&page);
    }
    assert_eq!(paged, ctx.registry.get_all_streams());
    assert_eq!(paged.len(), 120);
}

#[test]
fn test_duplicate_registration_does_not_grow_the_index() {
    let ctx = setup();
    register_n(&ctx, 2);
    let (sender, recipient, token) = (
        Address::generate(&ctx.env),
        Address::generate(&ctx.env),
        Address::generate(&ctx.env),
    );
    assert!(ctx
        .registry
        .try_register_stream(&100, &sender, &recipient, &token, &1)
        .is_err());
    assert_eq!(ctx.registry.stream_count(), 2);
}

/// Registers `n` streams with ids 100, 101, ... for paging tests.
fn register_n(ctx: &TestCtx, n: u64) {
    let sender = Address::generate(&ctx.env);
    let recipient = Address::generate(&ctx.env);
    let token = Address::generate(&ctx.env);
    for i in 0..n {
        ctx.registry
            .register_stream(&(100 + i), &sender, &recipient, &token, &1000);
    }
}
