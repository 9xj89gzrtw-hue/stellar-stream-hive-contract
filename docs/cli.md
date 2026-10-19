# Stellar CLI cheat sheet

Copy-paste `stellar contract invoke` commands for the common operations on the
Stream Hive contracts. Each command uses placeholders in `<angle brackets>`
for ids and addresses. Set them once as shell variables, then paste the
commands as written.

```bash
export NETWORK=testnet
export STREAM_ID=<stream contract id>        # C... (see DEPLOYMENTS.md)
export VAULT_ID=<vault contract id>          # C...
export REGISTRY_ID=<registry contract id>    # C...
export SENDER=<sender address>               # G...
export RECIPIENT=<recipient address>         # G...
export TOKEN=<token contract id>             # C... (the SAC for XLM, or any SEP-41 token)
```

Every write call takes `--source <identity>`. The identity must be the account
that the function authorizes, so it is the sender for `create_stream` and
`cancel_stream`, the recipient for `withdraw`, and the admin for `pause`.
Create identities with `stellar keys generate <name> --network testnet --fund`.

Amounts are in the token's base units (stroops for XLM, where 1 XLM =
10,000,000 stroops). Ledger numbers are absolute sequence numbers, not
durations.

Argument names follow the function signatures in
`contracts/*/src/lib.rs`. Use `--` to separate CLI flags from the contract
function and its arguments.

## Streams

### Create a stream

Locks `total_amount` in the vault and returns the new stream id. Signed by
`sender`. Fails with `Paused` while the contract is paused.

```bash
stellar contract invoke --id $STREAM_ID --source <sender identity> --network $NETWORK -- \
  create_stream \
  --sender $SENDER \
  --recipient $RECIPIENT \
  --token $TOKEN \
  --total_amount 100000000 \
  --start_ledger <start ledger> \
  --end_ledger <end ledger> \
  --cliff_ledger <cliff ledger> \
  --cancellable true
```

`cliff_ledger` must sit within `[start_ledger, end_ledger]`, and `end_ledger`
must be after `start_ledger`.

### Read a stream

```bash
stellar contract invoke --id $STREAM_ID --source <any identity> --network $NETWORK -- \
  get_stream --stream_id <stream id>
```

Other read-only calls that take the same `--stream_id` argument:

```bash
# Amount accrued so far, including what has already been withdrawn
stellar contract invoke --id $STREAM_ID --source <any identity> --network $NETWORK -- \
  streamed_amount --stream_id <stream id>

# Amount the recipient can withdraw right now (0 before the cliff)
stellar contract invoke --id $STREAM_ID --source <any identity> --network $NETWORK -- \
  withdrawable_amount --stream_id <stream id>

# Lifecycle status of the stream
stellar contract invoke --id $STREAM_ID --source <any identity> --network $NETWORK -- \
  get_status --stream_id <stream id>
```

To list streams, use the paged accessors. `offset` and `limit` are the paging
arguments, and `limit` is capped at 50:

```bash
stellar contract invoke --id $STREAM_ID --source <any identity> --network $NETWORK -- \
  get_streams_by_sender_page --sender $SENDER --offset 0 --limit 20

stellar contract invoke --id $STREAM_ID --source <any identity> --network $NETWORK -- \
  get_streams_by_recipient_page --recipient $RECIPIENT --offset 0 --limit 20
```

### Withdraw

Signed by `recipient`. `withdraw` takes an explicit amount, which must not
exceed `withdrawable_amount`:

```bash
stellar contract invoke --id $STREAM_ID --source <recipient identity> --network $NETWORK -- \
  withdraw \
  --recipient $RECIPIENT \
  --stream_id <stream id> \
  --amount <amount in base units>
```

`withdraw_max` withdraws everything currently withdrawable and returns the
amount withdrawn:

```bash
stellar contract invoke --id $STREAM_ID --source <recipient identity> --network $NETWORK -- \
  withdraw_max \
  --recipient $RECIPIENT \
  --stream_id <stream id>
```

### Cancel

Signed by `sender`. Only works when the stream was created with
`--cancellable true`. The unstreamed remainder returns to the sender. A
cancel before the cliff refunds the sender in full.

```bash
stellar contract invoke --id $STREAM_ID --source <sender identity> --network $NETWORK -- \
  cancel_stream \
  --sender $SENDER \
  --stream_id <stream id>
```

## Admin

These calls require the admin identity as `--source`. `pause` and `unpause`
take no arguments.

```bash
# Block create_stream. Withdrawals and cancellations keep working.
stellar contract invoke --id $STREAM_ID --source <admin identity> --network $NETWORK -- pause

# Resume create_stream
stellar contract invoke --id $STREAM_ID --source <admin identity> --network $NETWORK -- unpause

# Read the pause flag
stellar contract invoke --id $STREAM_ID --source <any identity> --network $NETWORK -- is_paused
```

To hand the admin role to a new address, run `transfer_admin` with the current
admin as `--source`. The new admin must also authorize the call, so it needs
its own signature on the same transaction:

```bash
stellar contract invoke --id $STREAM_ID --source <current admin identity> --network $NETWORK -- \
  transfer_admin --new_admin <new admin address>
```

## Vault and registry reads

```bash
# Locked balance for one stream
stellar contract invoke --id $VAULT_ID --source <any identity> --network $NETWORK -- \
  balance_of_stream --stream_id <stream id>

# Aggregate stats from the registry
stellar contract invoke --id $REGISTRY_ID --source <any identity> --network $NETWORK -- get_stats

# One page of registered stream ids
stellar contract invoke --id $REGISTRY_ID --source <any identity> --network $NETWORK -- \
  get_streams_page --offset 0 --limit 20
```

For the full function list and return types, see
[docs/stream.md](stream.md#function-reference),
[docs/vault.md](vault.md#function-reference) and
[docs/registry.md](registry.md#function-reference).
