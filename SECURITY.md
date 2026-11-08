# Security policy

## Supported versions

Only the latest commit on the `main` branch is supported. This project has
not been audited and is intended for Testnet use only. Do not use it on
Mainnet to hold real funds.

## Reporting a vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Email a description of the issue to:

mdeelive@gmail.com

Include:

- A description of the vulnerability and its impact.
- Steps to reproduce or a proof-of-concept (Testnet only, no real funds).
- Your suggested fix, if you have one.

You will receive an acknowledgement within 72 hours. We will coordinate a fix
and disclosure timeline with you.

## Known fixed vulnerabilities

See [`docs/SECURITY_NOTES.md`](docs/SECURITY_NOTES.md) for details.

| ID | Summary | Fixed in |
|----|---------|----------|
| CVE-2026-001 | Courier self-completion exploit via bearer secret | commit 1ca4b80 → this commit |

## Scope

- Smart contract logic (`contracts/escrow/`)
- Wallet integration and transaction signing (`web/src/lib/wallet.ts`)
- On-chain authorization model (`docs/AUTHORIZATION.md`)

## Out of scope

- Issues that require physical access to the user's device.
- Vulnerabilities in third-party dependencies (report those upstream).
- Denial-of-service attacks against Stellar network infrastructure.
