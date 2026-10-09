# Security Policy

These contracts custody user funds (through the vault), so security reports
are taken seriously.

## Reporting a vulnerability

Please **do not open a public issue** for a suspected vulnerability. Use
GitHub's private reporting instead:
**Security -> Report a vulnerability** on this repository.

Include the affected contract and function, a description of the impact, and
ideally a failing test that demonstrates it. We aim to acknowledge reports
within 72 hours.

## Scope

In scope: `contracts/vault`, `contracts/stream`, `contracts/registry`, and
the deployment script. Of particular interest:

- anything that lets a caller other than the configured stream contract move
  vault funds (see the vault's "security boundary" in `docs/vault.md`),
- any way for a recipient to withdraw more than has accrued, or for the sum of
  payouts and refunds to exceed a stream's deposit,
- anything that can trap funds in a stream (the `pause` switch is designed
  never to).

## Status

The contracts have **not** been externally audited. Do not use them with
mainnet funds you cannot afford to lose.
