# Deployments

This file records the on-chain addresses of every deployed Stellar Stream
Hive contract, per network.

## Testnet

_Deployment pending — run `scripts/deploy.sh` once the Stellar CLI and a
funded identity are available. Contract IDs will be recorded here._

| Contract | Contract ID | WASM hash | Deployed at |
|---|---|---|---|
| Vault | _pending_ | _pending_ | _pending_ |
| Stream | _pending_ | _pending_ | _pending_ |
| Registry | _pending_ | _pending_ | _pending_ |

## Mainnet

_Not yet deployed. Mainnet deployment will only follow a successful
testnet deployment, a completed audit, and explicit sign-off — this
project is a grant-stage prototype._

## How this file gets updated

`scripts/deploy.sh <network> <identity>` builds, optimizes and deploys all
three contracts in dependency order (vault -> stream -> registry), initializes
them, and then **verifies the wiring on-chain** (admins, the vault's trusted
stream contract, and that the stream contract starts unpaused) before
reporting success.

It writes `deployments/<network>.json` (contract ids, WASM sha256, admin,
timestamp) and prints a ready-to-paste table for this file plus the
environment lines for the frontend (`NEXT_PUBLIC_*_CONTRACT_ID`) and the API
(`STREAM_CONTRACT_ID`).

Deploying to mainnet additionally requires `CONFIRM_MAINNET=yes`; the
contracts have not been audited (see `SECURITY.md`).
