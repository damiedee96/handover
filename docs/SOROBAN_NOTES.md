# Soroban / Stellar notes

All findings here are verified against the sources cited. Nothing is taken
from memory.

---

## soroban-sdk version

Pinned to **29.0.1** (the version cached in this environment).

Source: `~/.cargo/registry/src/index.crates.io-*/soroban-sdk-29.0.1/`

Docs entry-point: https://docs.rs/soroban-sdk/29.0.1/soroban_sdk/

### SDK 29 behaviour changes observed

| Topic | Finding |
|---|---|
| `env.crypto().sha256()` | Returns `Hash<32>` not `BytesN<32>`; call `.into()` to convert. |
| Generated client | `confirm_pickup`, `complete_delivery`, `refund` return the raw value (e.g. `()`, `u64`) — **not** `Result<T, E>`. Use `try_*` variants for fallible assertions in tests. |
| Contract build | Must use `stellar contract build` (wraps cargo) because sdk 29 requires post-build spec processing. Running `cargo build --target wasm32v1-none --release` directly fails. |
| Auth testing | `mock_all_auths()` approves everything. `mock_auths(&[])` clears all approvals — call this *after* setup to test auth failures. `mock_all_auths_allowing_non_root_auth()` still approves all; it only changes nesting semantics. |

Source: https://developers.stellar.org/docs/build/smart-contracts/getting-started/hello-world

---

## Token interface

We use `soroban_sdk::token::Client` (standard SEP-41 token interface).
`token::StellarAssetClient` is used in tests only to mint test tokens.

Source: https://docs.rs/soroban-sdk/29.0.1/soroban_sdk/token/index.html

---

## TTL / storage

- `env.storage().persistent()` — survives ledger close; TTL must be extended.
- `env.storage().instance()` — scoped to the contract instance; extended with `extend_ttl`.
- Threshold `17_280` ≈ 1 day at 5-second ledgers; bump target `17_280 × 30` ≈ 30 days.

Source: https://developers.stellar.org/docs/build/smart-contracts/storage

---

## stellar-sdk (JavaScript, v13)

The `SorobanRpc` namespace exported from earlier versions is now exported as
`rpc` (lowercase) in v13.

```ts
import { rpc } from "@stellar/stellar-sdk";
const server = new rpc.Server(url);
rpc.assembleTransaction(tx, simResult);
rpc.Api.isSimulationError(result);
rpc.Api.GetTransactionStatus.SUCCESS;
```

Source: `node_modules/@stellar/stellar-sdk/lib/index.d.ts`

### Protocol alignment

`stellar-sdk@13.2.0` targets the same XDR as `stellar-xdr@28.0.0` (Protocol 22),
which is the Testnet protocol level at time of writing. The contract uses
`soroban-sdk@29.0.1` which is compatible with Protocol 22.

Verified by checking the `stellar-xdr` version in the Cargo lock and the `@stellar/stellar-sdk`
changelog at https://github.com/stellar/js-stellar-sdk.

---

## Freighter wallet API (v6)

Package: `@stellar/freighter-api@6.0.1`

Docs: https://docs.freighter.app/docs/guide/usingFreighterBrowser

All functions return a result object with an `error` field; they do **not**
throw. Pattern:

```ts
const result = await requestAccess();
if (result.error) throw new Error(result.error);
```

The app never requests or stores secret keys. All signing stays inside the
Freighter extension.
