# Architecture decisions

## Rust / contract

| Dependency | Version | Reason |
|---|---|---|
| `soroban-sdk` | `29.0.1` | Latest version cached offline in this environment. |
| Rust edition | `2021` | Current stable edition; required by soroban-sdk. |
| `wasm32v1-none` target | pre-installed | Required for `stellar contract build`. |

## TypeScript / web

| Dependency | Version | Reason |
|---|---|---|
| `vite` | `6.4.5` | Vite 8 requires `lightningcss ^1.33` which was not cached. Vite 6 uses `lightningcss ^1.21` (optional peer dep) and is the current stable major. |
| `react` / `react-dom` | `19.1.8` | Latest stable React 19; full support for modern hooks. |
| `react-router-dom` | `6.30.1` | Declarative routing for the three-page SPA. |
| `qrcode.react` | `4.2.0` | Well-maintained QR component; renders as SVG for accessibility. |
| `@stellar/stellar-sdk` | `13.2.0` | Current JS SDK; provides `rpc.Server`, `TransactionBuilder`, `xdr`, etc. |
| `@stellar/freighter-api` | `6.0.1` | Official Freighter extension API; enables wallet signing without touching secret keys. Chosen over `@creit-tech/stellar-wallets-kit` because the wallets-kit npm package was not available in the offline cache. |
| `typescript` | `5.8.3` | Latest stable; strict mode enabled. |

## Security choices

- Secret is generated with `crypto.getRandomValues` (CSPRNG).
- Only SHA-256 of the secret is stored on-chain; the plaintext never hits a server.
- The share link encodes the secret in the URL **fragment** (`#`), which browsers never send to servers.
- No custom cryptography; SHA-256 via the Soroban host and the Web Crypto API only.

## Numeric precision

Token amounts are stored as `i128` in the contract. The UI converts between
user-visible decimal and stroops (`×1e7`) consistent with XLM denomination.
For tokens with different decimals, operators should adjust this in `SenderPage`.

## Network configuration

Network, RPC URL, contract address, and token address all come from Vite env
vars (`VITE_*`). `.env.example` documents every variable. Real values are
never committed.
