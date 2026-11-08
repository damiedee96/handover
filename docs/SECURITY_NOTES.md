# Security notes

## CVE-2026-001 — Courier self-completion exploit

### Summary

A courier who obtained the completion secret (the bearer token used in the
original design) could call `confirm_pickup` and then `complete_delivery`
immediately, without ever delivering the parcel, and collect the full escrow.

### Original design

The original contract stored a `secret_hash` on-chain. The flow was:

1. Sender creates the delivery and sends the plaintext secret to the recipient.
2. Courier calls `confirm_pickup` (releases pickup fraction).
3. Courier calls `complete_delivery(secret)` — passing the plaintext secret —
   to release the remainder.

The intended model was that the recipient would only reveal the secret to the
courier at physical handoff. However, the contract itself had no way to enforce
this: **the secret is a bearer token**. Anyone who holds it — including the
courier, if they obtained it by other means — could call `complete_delivery`
at any time after `confirm_pickup`.

In practice the courier could:

1. Intercept or guess the secret before delivery.
2. Call `confirm_pickup`.
3. Immediately call `complete_delivery` with the secret.
4. Walk away with the full escrow without delivering.

The contract could not distinguish a legitimate post-delivery call from this
exploit sequence.

### Fix

The secret-based completion mechanism was removed entirely. The contract now
has a `recipient` field. Completion is triggered by `confirm_receipt(id)`,
which requires the **recipient's on-chain signature** (enforced by
`recipient.require_auth()`).

Because the recipient's private key is never shared with the courier, the
courier cannot call `confirm_receipt` on their own. The separation is:

- **Courier** controls `confirm_pickup` → collects the pickup fraction.
- **Recipient** controls `confirm_receipt` → releases the remainder to the
  courier, but only the recipient can initiate this.

This eliminates the exploit: a courier who calls `confirm_pickup` and then
attempts `confirm_receipt` without the recipient's key will be rejected by the
contract's `require_auth` check.

### Test coverage

`test_exploit_courier_cannot_self_complete` in `contracts/escrow/src/test.rs`
reproduces the exact exploit sequence and asserts it is rejected.

`test_sender_cannot_confirm_receipt` verifies that the sender also cannot
trigger completion.

`test_auth_failure_confirm_receipt` verifies that confirm_receipt fails when
no valid auth is present.

### Removal of `WrongSecret` error code

Error variant `WrongSecret = 6` has been removed from the `Error` enum as it
is no longer needed. The discriminant `6` is retired; do not reuse it in
future error variants to avoid confusion with on-chain error data from
pre-fix contract instances.

### Impact on the web app

- `SenderPage` now collects the recipient's Stellar address and passes it to
  `create_delivery`. No secret is generated client-side.
- A new `RecipientPage` (`/recipient`) allows the recipient to connect their
  wallet and call `confirm_receipt`.
- The share link for the recipient carries only the delivery id; the recipient's
  wallet is the credential.
- `CourierPage` no longer has a secret input field.

### Protocol invariants after the fix

| Who | Can call | Cannot call |
|-----|----------|-------------|
| Sender | `create_delivery`, `refund` | `confirm_pickup`, `confirm_receipt` |
| Courier | `confirm_pickup` | `confirm_receipt`, `refund` |
| Recipient | `confirm_receipt` | `confirm_pickup`, `refund` |
| Anyone | `get_delivery` | — |
