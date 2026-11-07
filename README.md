# Handover

On-chain escrow for in-person deliveries, built on [Stellar Soroban](https://stellar.org/soroban).

A sender locks tokens in the escrow contract. The courier confirms pickup (receiving a configurable fraction immediately) and completes the delivery by presenting the correct secret code, releasing the remainder. If the courier doesn't deliver before expiry, the sender can reclaim the funds.

## Repository layout

```
contracts/escrow/   Soroban smart contract (Rust)
web/                React + Vite + TypeScript front-end
docs/               Design notes and decisions
```

## Contract

Five public functions:

| Function | Caller | Description |
|---|---|---|
| `create_delivery` | Sender | Lock tokens; returns delivery id |
| `confirm_pickup` | Courier | Release pickup fraction |
| `complete_delivery` | Courier | Verify secret; release remainder |
| `refund` | Sender | Reclaim funds after expiry |
| `get_delivery` | Anyone | Read delivery record |

### Build

Requires the [Stellar CLI](https://developers.stellar.org/docs/tools/stellar-cli) and the `wasm32v1-none` Rust target.

```sh
rustup target add wasm32v1-none
stellar contract build
```

### Test

```sh
cargo test
```

27 tests cover the full lifecycle, error paths, auth failures, and edge cases.

## Web app

Sender page — create a delivery, get a QR code and share link for the courier.  
Courier page — confirm pickup and complete delivery using the secret.  
Status page — look up any delivery; refund after expiry.

Wallet connection uses the [Freighter](https://freighter.app) browser extension.
The app never handles secret keys.

### Setup

```sh
cd web
cp .env.example .env
# Edit .env with your contract id, token, RPC url, and network
npm install
npm run dev
```

### Typecheck and build

```sh
npm run typecheck
npm run build
```

## Docs

- [`docs/DECISIONS.md`](docs/DECISIONS.md) — dependency and architecture choices
- [`docs/SOROBAN_NOTES.md`](docs/SOROBAN_NOTES.md) — verified SDK behaviour
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — planned work

## License

MIT
