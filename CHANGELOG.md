# Changelog

All notable changes to the Stellar Stream Hive contracts. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- **Stream:** `pause` / `unpause` / `is_paused`. Pausing blocks only
  `create_stream`; withdrawals and cancellations always keep working so
  funds are never trapped.
- **Stream:** `transfer_admin`, requiring authorization from both the old and
  new admin.
- **Stream:** `count_streams_by_sender|recipient` and paginated
  `get_streams_by_sender|recipient_page` (page size capped at 50) so
  addresses with many streams stay within Soroban resource limits.
- **Stream:** new `Paused` error (code 13) and `Pause` / `Unpause` /
  `AdminTransfer` events.
- Math invariant tests: monotonic accrual, withdrawals never exceed the
  deposit, rounding never favours the recipient, cancel-before-cliff pays
  nothing.
- CI: `cargo fmt --check` and clippy (`-D warnings`) job.
- `SECURITY.md` with a private disclosure process.

### Changed
- **Registry:** stream ids are now stored one ledger entry per registration
  index (`StreamCount` + `StreamIdAt`) instead of a single growing `Vec`, so
  registration stays O(1) and cannot hit the ledger-entry size limit. Added
  `stream_count` and `get_streams_page` (page size capped at 50). Storage
  layout change: only safe before a first deployment.
- **Stream:** the `create` event now also carries `cliff_ledger` and
  `cancellable`, so an indexer can rebuild a stream from events alone.
  **Breaking for event consumers** -- the API repo is updated to match.

## [0.1.0]
- Initial vault, stream and registry contracts.
