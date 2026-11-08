# Authorization — who can call what

This document describes, in plain language, which account is allowed to call
each contract function and what happens if the wrong account tries.

---

## create_delivery

**Who:** The **sender** — the account that will pay for the delivery.

The sender's address is passed as the first argument. The contract calls
`sender.require_auth()` before transferring tokens. If the transaction is not
signed by the sender, the call is rejected before any tokens move.

---

## confirm_pickup

**Who:** The **courier** — the account recorded in the delivery.

The courier's address is stored in the delivery record at creation time. The
contract calls `courier.require_auth()`. Nobody else (not even the sender or
recipient) can call this function.

Additional guard: if the current ledger sequence is past `expiry_ledger`, the
call is also rejected, even if the courier's auth is valid.

---

## confirm_receipt

**Who:** The **recipient** — the account that receives the parcel.

The recipient's address is stored in the delivery record at creation time. The
contract calls `recipient.require_auth()`. The courier **cannot** call this
function. This is the primary security guarantee of the protocol: the courier
can collect their pickup fraction (via `confirm_pickup`) but cannot release
the remainder to themselves without the recipient's signature.

Additional guard: the current ledger must not be past `expiry_ledger`.

---

## refund

**Who:** The **sender** — the account that created the delivery.

The sender calls this after `expiry_ledger` has passed. The contract calls
`sender.require_auth()`. The call is rejected if the ledger is still at or
before `expiry_ledger`, or if the delivery is already `Completed` or
`Refunded`.

---

## get_delivery

**Who:** Anyone.

Read-only simulation. No auth required. No state changes.

---

## Summary table

| Function         | Required signer | Extra guard |
|------------------|-----------------|-------------|
| create_delivery  | sender          | amount > 0, pickup_bps ≤ 5000, expiry > now |
| confirm_pickup   | courier         | status == Created, ledger ≤ expiry |
| confirm_receipt  | recipient       | status == PickedUp, ledger ≤ expiry |
| refund           | sender          | ledger > expiry, status ∈ {Created, PickedUp} |
| get_delivery     | —               | — |

---

## Why the courier cannot complete the delivery alone

This is the fix for CVE-2026-001. See [`docs/SECURITY_NOTES.md`](SECURITY_NOTES.md).

In the old design a `complete_delivery(secret)` function existed that only
required the courier's auth plus a bearer secret. Any party who held the
secret — including a courier who intercepted it — could call it immediately
after pickup.

In the new design, `confirm_receipt` requires the recipient's signature.
The recipient's private key is never shared with the courier. A courier who
calls `confirm_pickup` and then attempts `confirm_receipt` with only their own
key will be rejected.
