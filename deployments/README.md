# Deployments

Each `<network>.json` file in this directory is the machine-readable record of
one network's deployment. It is written by `scripts/deploy.sh <network> <identity>`
and is read by humans and scripts. The human-readable summary lives in
[`DEPLOYMENTS.md`](../DEPLOYMENTS.md).

**The file is overwritten on every deployment.** Re-running the script on the
same network deploys fresh contract instances with new ids and replaces the
previous file. The prior ids are only kept in git history and in
`DEPLOYMENTS.md`, so update that file when you deploy.

## Fields

```json
{
  "network": "testnet",
  "deployed_at": "2026-10-12T09:30:26Z",
  "admin": "GBLEJOG7B6ONTCL4F3T7F5G5NVYVKLLEMTV2C6XVZ7BF2ML4TUQVVZHI",
  "contracts": {
    "vault":    { "id": "CB67...", "wasm_sha256": "3f66..." },
    "stream":   { "id": "CCYM...", "wasm_sha256": "317e..." },
    "registry": { "id": "CA3M...", "wasm_sha256": "ee85..." }
  }
}
```

| Field | Meaning |
|---|---|
| `network` | The network name passed to the script (`testnet`, `futurenet`, `local` or `mainnet`). |
| `deployed_at` | UTC timestamp in ISO 8601 format (`YYYY-MM-DDTHH:MM:SSZ`) recorded when the script ran. |
| `admin` | The Stellar account (`G...` address) that was set as admin on all three contracts. It is the address of the deployer identity. |
| `contracts.vault.id` | The vault contract's address (`C...`). The vault holds the funds. |
| `contracts.vault.wasm_sha256` | SHA-256 of the optimized vault WASM that was uploaded. Use it to check that the on-chain code matches a local build. |
| `contracts.stream.id` | The stream contract's address (`C...`). It is the only caller the vault trusts. |
| `contracts.stream.wasm_sha256` | SHA-256 of the optimized stream WASM. |
| `contracts.registry.id` | The registry contract's address (`C...`). It is a read-only index. |
| `contracts.registry.wasm_sha256` | SHA-256 of the optimized registry WASM. |

The hashes are computed after `stellar contract optimize` runs, so they match
the optimized `*.optimized.wasm` artifacts and not the raw release builds.

## Related

- [`DEPLOYMENTS.md`](../DEPLOYMENTS.md) is the human-readable record of each
  network's contracts, with explorer links and the smoke-test transaction.
- [`scripts/deploy.sh`](../scripts/deploy.sh) describes how the file is generated and
  verifies the wiring before it is written.
