# Limitations

This document describes known limitations and trust assumptions. Read it
before using Handover in any context beyond personal experimentation on
Testnet.

---

## Testnet only

The contract and web app are configured for Stellar Testnet. The app enforces
this at startup: if `VITE_STELLAR_NETWORK` is not `TESTNET` the app refuses to
run. Testnet tokens have no real-world value. **Do not deploy this contract to
Mainnet or use it to hold real funds.**

---

## Not audited

This code has not been reviewed by an independent security researcher. There
may be bugs in the contract logic, the secret-handling code, or the
transaction-building code. Use at your own risk.

---

## TTL and storage restore

Soroban persistent storage entries expire if their TTL is not extended. The
contract bumps the TTL of each delivery record and the counter on every write,
targeting ~30 days. If no activity occurs within that window the entry
becomes "archived" and must be restored using the Stellar network's
`RestoreFootprintOp` before it can be accessed again. The web app does not
implement restore; you would need to use the Stellar CLI.

---

## Bearer-secret trust

The secret is a 128-bit random value. Whoever possesses the plaintext secret
can cause `complete_delivery` to succeed, provided they also control the
courier's Stellar key. The system assumes the recipient keeps the secret
confidential and only reveals it at physical handoff. There is no
cryptographic proof that handoff actually occurred.

---

## Token trust

The contract works with any SEP-41-compatible token. The token contract itself
is trusted: a malicious token could block transfers, manipulate balances, or
revert in unexpected ways. Only use tokens whose contracts you trust.

---

## Ledger-based expiry

Expiry is measured in ledger sequence numbers, not wall-clock time. Stellar
targets ~5 seconds per ledger, but this is not guaranteed. Expiry in "days"
shown in the UI is an approximation based on 17,280 ledgers per day.

---

## Secret known to the sender

The sender generates the secret and therefore knows it. A malicious sender
could share the secret with the courier directly, bypassing the recipient.
The protocol assumes the sender is honest.

---

## No dispute resolution

The contract has no arbitration mechanism. If the courier and recipient
disagree about whether handoff occurred, there is no on-chain recourse beyond
waiting for expiry and refunding.
