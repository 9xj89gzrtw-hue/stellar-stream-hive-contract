# Events

Every contract event in the Stellar Stream Hive contracts, in one place. Use
this page to index the protocol: it lists the topics and data fields for the
Stream, Vault and Registry contracts.

## How to read this

Each event is a typed `#[contractevent]` struct defined in that contract's
`src/events.rs`. The struct determines the topics and the data:

- **Topic 0** is the struct name converted to snake_case (`Create` becomes
  `create`, `SetStreamContract` becomes `set_stream_contract`).
- **Topic 1** is present only when the struct marks a field with `#[topic]`.
  It is the `stream_id` for every stream and vault event that has one. The
  admin-only events (`Pause`, `Unpause`, `AdminTransfer`) and
  `SetStreamContract` have no second topic.
- **Data** is the remaining fields in declaration order. Structs with
  `data_format = "vec"` publish their data as a tuple. Structs with
  `data_format = "single-value"` publish one bare value.

The "Data" column below lists fields in that same order.

## Stream contract

Source: [`contracts/stream/src/events.rs`](../contracts/stream/src/events.rs)

| Event struct | Topics | Data | Emitted when |
|---|---|---|---|
| `Create` | `("create", stream_id)` | `(sender, recipient, token, total_amount, start_ledger, end_ledger, cliff_ledger, cancellable)` | `create_stream` succeeds |
| `Withdraw` | `("withdraw", stream_id)` | `(recipient, amount)` | `withdraw` or `withdraw_max` succeeds |
| `Cancel` | `("cancel", stream_id)` | `(sender, streamed_now, remainder)` | `cancel_stream` succeeds |
| `Pause` | `("pause")` | `(admin)` | `pause` succeeds |
| `Unpause` | `("unpause")` | `(admin)` | `unpause` succeeds |
| `AdminTransfer` | `("admin_transfer")` | `(previous_admin, new_admin)` | `transfer_admin` succeeds |

The `Create` event carries every field an indexer needs to rebuild a stream,
so no follow-up read is required. See [docs/stream.md](stream.md#events) for
the reasoning behind that.

## Vault contract

Source: [`contracts/vault/src/events.rs`](../contracts/vault/src/events.rs)

| Event struct | Topics | Data | Emitted when |
|---|---|---|---|
| `Deposit` | `("deposit", stream_id)` | `(from, token, amount)` | `deposit` succeeds |
| `Release` | `("release", stream_id)` | `(to, token, amount)` | `release` succeeds |
| `Refund` | `("refund", stream_id)` | `(to, token, amount)` | `refund` succeeds |
| `SetStreamContract` | `("set_stream_contract")` | `new_contract` (single value) | `set_stream_contract` succeeds |

## Registry contract

Source: [`contracts/registry/src/events.rs`](../contracts/registry/src/events.rs)

| Event struct | Topics | Data | Emitted when |
|---|---|---|---|
| `Register` | `("register", stream_id)` | `amount` (single value) | `register_stream` succeeds |
| `Update` | `("update", stream_id)` | `withdrawn` (single value) | `update_stream` succeeds |

The registry uses single-value data, so the event carries only the amount, not
a tuple.

## Filtering by topic

Topics are what an indexer filters on. To find every event for one stream,
match the second topic against that stream's id. To find every `create`
event, match the first topic against `create`. Because the first topic is
always the event name, a filter on a single name returns events from every
stream.
