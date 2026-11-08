# Handover

On-chain escrow for in-person deliveries, built on [Stellar Soroban](https://stellar.org/soroban). **Testnet only. Not audited. Do not use for real funds.**

The sender locks tokens in the contract. The courier confirms pickup and receives a configurable fraction immediately. The recipient holds a secret code; at physical handoff they reveal it to the courier, who types it into the app to release the remainder. If delivery does not happen before expiry, the sender can reclaim the funds.

See [`docs/LIMITATIONS.md`](docs/LIMITATIONS.md) for a full list of trust assumptions.

---

## Repository layout

```
contracts/escrow/          Soroban smart contract (Rust)
web/                       React + Vite + TypeScript front-end
docs/                      Design notes and decisions
.github/workflows/ci.yml   GitHub Actions CI
```

---

## Contract

Five public functions — see [`docs/AUTHORIZATION.md`](docs/AUTHORIZATION.md) for who can call each one.

| Function | Caller | Description |
|---|---|---|
| `create_delivery` | Sender | Lock tokens; returns delivery id |
| `confirm_pickup` | Courier | Release pickup fraction (rejected after expiry) |
| `confirm_receipt` | Recipient | Release remainder to courier (wallet auth required) |
| `refund` | Sender | Reclaim funds after expiry |
| `get_delivery` | Anyone | Read delivery record |

### Build the contract

Requires [Stellar CLI ≥ 25.2](https://developers.stellar.org/docs/tools/stellar-cli) and the `wasm32v1-none` Rust target.

```sh
rustup target add wasm32v1-none
stellar contract build
# Output: target/wasm32v1-none/release/handover_escrow.wasm
```

### Test

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
# Expected: 33 tests pass, 0 fail
```

---

## Deploy to Testnet

> **Note:** The following steps require `stellar` CLI and network access.
> They were not executed in the build environment (Stellar CLI was not installed
> there). The commands are correct per the
> [Stellar CLI docs](https://developers.stellar.org/docs/tools/stellar-cli)
> but you must run them yourself.

### 1. Create a named identity

```sh
stellar keys generate --global alice --network testnet
stellar keys address alice
# Prints: G<publickey>
```

### 2. Fund it from the Testnet faucet

```sh
stellar keys fund alice --network testnet
```

### 3. Deploy the contract

```sh
stellar contract deploy \
  --wasm target/wasm32v1-none/release/handover_escrow.wasm \
  --source alice \
  --network testnet
# Prints: C<contractid>  — save this as VITE_ESCROW_CONTRACT_ID
```

### 4. Get a Testnet token contract id

You can deploy the Stellar Asset Contract (SAC) for Testnet XLM:

```sh
stellar contract id asset --asset native --network testnet
# Prints a contract id for the native XLM SAC — use as VITE_TOKEN_CONTRACT_ID
```

Alternatively, issue your own Testnet asset and wrap it:

```sh
stellar contract id asset \
  --asset USDC:G<issuer-address> \
  --network testnet
```

### 5. Fill in the `.env` file

```sh
cd web
cp .env.example .env
```

Edit `.env`:

```
VITE_STELLAR_NETWORK=TESTNET
VITE_RPC_URL=https://soroban-testnet.stellar.org
VITE_ESCROW_CONTRACT_ID=C<your-contract-id>
VITE_TOKEN_CONTRACT_ID=C<your-token-id>
```

### 6. Run the app

```sh
npm install
npm run dev
# Open http://localhost:5173
```

Install the [Freighter wallet extension](https://freighter.app) in your browser
and switch it to Testnet before connecting.

---

## Web app

**Sender page** — create a delivery (courier + recipient addresses, amount, pickup %, expiry). Receive a link for the recipient and a lookup link for the courier.

**Courier page** — look up a delivery by id; confirm pickup.

**Recipient page** — connect wallet; confirm receipt after the parcel arrives. Wallet signature releases remainder to the courier.

**Status page** — look up any delivery; refund unreleased funds after expiry.

The app never handles secret keys. All signing is done inside Freighter.

### Typecheck, test, and build

```sh
cd web
npm run typecheck   # 0 errors expected
npm run test        # 11 tests pass
npm run build       # outputs dist/
```

---

## Docs

| File | Contents |
|---|---|
| [`docs/SECURITY_NOTES.md`](docs/SECURITY_NOTES.md) | CVE-2026-001 vulnerability and fix |
| [`docs/AUTHORIZATION.md`](docs/AUTHORIZATION.md) | Who can call each function |
| [`docs/LIMITATIONS.md`](docs/LIMITATIONS.md) | Known limitations and trust assumptions |
| [`docs/DECISIONS.md`](docs/DECISIONS.md) | Pinned versions and rationale |
| [`docs/SOROBAN_NOTES.md`](docs/SOROBAN_NOTES.md) | Verified SDK behaviour with sources |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | Planned but unbuilt features |

---

## License

MIT — see [`LICENSE`](LICENSE).
