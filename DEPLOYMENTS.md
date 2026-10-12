# Deployments

This file records the on-chain addresses of every deployed Stellar Stream
Hive contract, per network.

## Testnet

Deployed 2026-10-12 with `scripts/deploy.sh testnet deployer`. Admin (all three
contracts): `GBLEJOG7B6ONTCL4F3T7F5G5NVYVKLLEMTV2C6XVZ7BF2ML4TUQVVZHI`.
Machine-readable copy: [`deployments/testnet.json`](deployments/testnet.json).

| Contract | Contract ID | WASM sha256 | Deployed at |
|---|---|---|---|
| Vault | [`CB67RYDOOIOR2YZERK77URMXDCOLDAVOFELZERDCH562OH2NMP2UNG2N`](https://stellar.expert/explorer/testnet/contract/CB67RYDOOIOR2YZERK77URMXDCOLDAVOFELZERDCH562OH2NMP2UNG2N) | `3f660d81c46d412686c2908b1f0111ae5d9d44a972964487e89b8de829533d47` | 2026-10-12T09:30:26Z |
| Stream | [`CCYM2PIMNHOJKWRV677QQNFUMNU2JKEHXTQKK6D3D7JF2UTEB5JNINYY`](https://stellar.expert/explorer/testnet/contract/CCYM2PIMNHOJKWRV677QQNFUMNU2JKEHXTQKK6D3D7JF2UTEB5JNINYY) | `317e697348e4144c04aafc65d4be4da7bb6a9d3dffb270967c440cd48cd8a581` | 2026-10-12T09:30:26Z |
| Registry | [`CA3MEGSTTI576GDVRFXZ2YJGGO3LAKOPNPGZ34F46FEDDHDCOWYQTNMG`](https://stellar.expert/explorer/testnet/contract/CA3MEGSTTI576GDVRFXZ2YJGGO3LAKOPNPGZ34F46FEDDHDCOWYQTNMG) | `ee857043d09cc82d8c07cb3305a61fb2c689edec1caea2b3ae646bb5d1f0ef54` | 2026-10-12T09:30:26Z |

The deploy script verified on-chain that the vault and registry admins are the
deployer, that the vault trusts the Stream contract, and that Stream starts
unpaused.

**Smoke test.** A real 10 XLM stream (id 1) was created on this deployment
([transaction](https://stellar.expert/explorer/testnet/tx/d22931b46cc4db0edfa23b8ff06c90b85eec748f3b3e5f521cc2aee223852ec8)):
the funds moved into the vault, the `create` event carried all eight fields,
and `get_stream` returned the expected record.

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
