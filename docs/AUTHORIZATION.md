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
contract calls `courier.require_auth()`. Nobody else (not even the sender) can
call this function.

Additional guard: if the current ledger sequence is past `expiry_ledger`, the
call is also rejected, even if the courier's auth is valid.

---

## complete_delivery

**Who:** The **courier** — the same account that called `confirm_pickup`.

The courier must also supply the plaintext **secret** that was given to the
recipient at creation time. The contract hashes the secret with SHA-256 and
compares it to the stored `secret_hash`. If the hashes do not match the call
is rejected.

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

This is a read-only simulation. No auth is required. No state changes.

---

## Summary table

| Function           | Required signer  | Extra guard                        |
|--------------------|------------------|------------------------------------|
| create_delivery    | sender           | amount > 0, pickup_bps ≤ 5000, expiry > now |
| confirm_pickup     | courier          | status == Created, ledger ≤ expiry |
| complete_delivery  | courier          | status == PickedUp, ledger ≤ expiry, SHA-256(secret) == hash |
| refund             | sender           | ledger > expiry, status ∈ {Created, PickedUp} |
| get_delivery       | —                | —                                  |
