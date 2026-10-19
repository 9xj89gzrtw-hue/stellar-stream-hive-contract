# Registry Contract

`contracts/registry` — crate name `stellar-stream-hive-registry`.

## Overview

The registry is an on-chain index of every stream created across the
protocol, plus running aggregate statistics. It exists so that
dashboards, block explorers, and other contracts can answer questions like
"how many streams exist," "how much value is actively streaming," or "give
me every active stream id" without enumerating the stream contract's
per-address indexes one sender/recipient at a time.

The registry **does not custody funds** and is **not in the trust path**
for any transfer. It is a read model, not a source of truth — the stream
contract's own storage remains authoritative for any given stream's real
state.

## Data model

```rust
pub struct RegistryStats {
    pub total_streams: u64,
    pub active_streams: u64,
    pub total_streamed: i128,
    pub total_withdrawn: i128,
}
```

- `total_streams` / `active_streams` — simple counters, incremented on
  `register_stream` and adjusted on every `update_stream` status
  transition into/out of `Active`.
- `total_streamed` — a **lifetime, cumulative** sum of every stream's
  `amount` at registration time. This is a "total value ever committed to
  streaming" metric, not a live balance — it only ever goes up.
- `total_withdrawn` — cumulative sum of withdrawn amounts, updated by the
  delta (`new_withdrawn - old_withdrawn`) each time `update_stream` is
  called, since `update_stream` reports an absolute withdrawn total rather
  than an incremental amount.

Internally, each stream is also kept as a `StreamRecord` (sender,
recipient, token, amount, withdrawn, status) so `get_stream_record` and
`get_active_streams` can be served directly from the registry's own
storage.

## Function reference

### `initialize(env, admin)`
One-time setup, requires `admin.require_auth()`. Initializes `RegistryStats`
to all zeros.

### `register_stream(env, stream_id, sender, recipient, token, amount)`
Records a newly created stream. New streams are recorded with status
`Active` by convention (this contract has no visibility into a stream's
`start_ledger`/`cliff_ledger`, since those aren't part of this function's
signature — `update_stream` can correct the status later, e.g. back to
`Pending` if that distinction matters to a given deployment). Fails with
`AlreadyRegistered` if `stream_id` was already registered.

### `update_stream(env, stream_id, withdrawn, status)`
Updates a stream's withdrawn amount and status, adjusting
`active_streams` and `total_withdrawn` accordingly. Fails with
`StreamNotFound` if the id was never registered.

### `get_all_streams(env) -> Vec<u64>`
Every registered stream id, in registration order. Reads one ledger entry
per stream, so on a large registry prefer the paged accessors below.

### `stream_count(env) -> u32`
Number of streams ever registered.

### `get_streams_page(env, offset, limit) -> Vec<u64>`
A registration-ordered page of ids. `limit` is capped at `MAX_PAGE_SIZE`
(50); an `offset` past the end returns an empty vec.

### `get_stats(env) -> RegistryStats`
The current aggregate snapshot.

### `get_active_streams(env) -> Vec<u64>`
Ids of every stream whose last-known status is `Active`.

### `get_stream_record(env, stream_id) -> StreamRecord`
Read-only accessor for a single stream's registry-side summary.

## Events

| Topic | Payload | Emitted when |
|---|---|---|
| `("register", stream_id)` | `amount` | `register_stream` succeeds |
| `("update", stream_id)` | `withdrawn` | `update_stream` succeeds |

## Security notes: why write access here is intentionally open

Unlike the vault's `release`/`refund`, `register_stream` and
`update_stream` take **no caller parameter** — this matches the interface
this contract is built against. Without a caller argument, there is no
address for this contract to run `require_auth()` against to verify "only
the stream contract may call this," the way the vault verifies its caller
against a stored `stream_contract` address (see
[vault.md](./vault.md#the-security-boundary)).

This is a deliberate, documented trade-off rather than an oversight:

- The registry never moves funds. A malicious or buggy call to
  `register_stream`/`update_stream` can, at worst, corrupt the analytics
  this contract reports (an inflated `total_streams`, a fabricated stream
  id, an incorrect `active_streams` count). It cannot drain the vault,
  cannot alter a real stream's actual accrual, and cannot let anyone
  withdraw funds they aren't owed — the stream and vault contracts never
  read from the registry.
- The intended deployment calls these functions from the stream contract
  (or a trusted off-chain indexer) as part of its own `create_stream`/
  `withdraw`/`cancel_stream` flows.

**Hardening path for a future version**: add an explicit `caller: Address`
parameter to both functions (mirroring the vault's `release`/`refund`
pattern), store an `authorized_writer` address at `initialize`, and reject
any call where `caller != authorized_writer`. This was left out of v1 to
keep the function signatures matching the contract's specified interface
exactly; it is a straightforward, backwards-incompatible follow-up.

## Errors

| Code | Name | Meaning | Returned by |
|---|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` called twice | `initialize` |
| 2 | `NotInitialized` | called before `initialize` | `get_admin` |
| 3 | `InvalidAmount` | amount argument was negative | `register_stream`, `update_stream` |
| 4 | `AlreadyRegistered` | `stream_id` was already registered | `register_stream` |
| 5 | `StreamNotFound` | no stream with that id is registered | `update_stream`, `get_stream_record` |
| 6 | `MathOverflow` | checked arithmetic overflowed or underflowed | `register_stream`, `update_stream` |

The codes come from `contracts/registry/src/error.rs`. Errors are returned as
Soroban contract errors, so a client sees the numeric code.

## Storage layout

- `Admin` — instance storage.
- `StreamCount` — instance storage, number of registered streams.
- `StreamIdAt(index)` — persistent storage, the id registered at that index.
  One entry per stream (rather than one growing `Vec`) keeps registration
  O(1) and well clear of the ledger-entry size limit.
- `Record(stream_id)` — persistent storage, that stream's `StreamRecord`.
- `Stats` — instance storage, the current `RegistryStats`.
