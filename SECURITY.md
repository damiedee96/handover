# Security policy

## Supported versions

Only the latest commit on the `main` branch is supported. This project has
not been audited and is intended for Testnet use only. Do not use it on
Mainnet to hold real funds.

## Reporting a vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Email a description of the issue to the maintainer. Include:

- A description of the vulnerability and its impact.
- Steps to reproduce or a proof-of-concept (Testnet only).
- Your suggested fix, if you have one.

You will receive an acknowledgement within 72 hours. We will coordinate a fix
and disclosure timeline with you.

## Scope

- Smart contract logic (`contracts/escrow/`)
- Secret generation and transmission in the web app (`web/src/lib/secret.ts`)
- Wallet integration and transaction signing (`web/src/lib/wallet.ts`)

## Out of scope

- Issues that require physical access to the user's device.
- Vulnerabilities in third-party dependencies (report those upstream).
- Denial-of-service attacks against Stellar network infrastructure.
