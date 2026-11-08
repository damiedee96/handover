# Contributing

## Before you start

- Read `docs/LIMITATIONS.md` — understand what the project intentionally does not do.
- This repository is **Testnet only**. Do not add Mainnet support.
- No custom cryptography. Use the Soroban host crypto or the Web Crypto API only.
- No frontend frameworks beyond what is already present (React, Vite).
- No new server-side components, analytics, or external services.

## Workflow

1. Fork and create a branch named `fix/<topic>` or `feat/<topic>`.
2. Keep commits small and focused: one logical change per commit.
3. For contract changes, run `cargo fmt && cargo clippy -- -D warnings && cargo test`.
4. For web changes, run `npm run typecheck && npm run test && npm run build`.
5. All checks must pass before opening a PR.
6. Document any new Soroban SDK behaviour in `docs/SOROBAN_NOTES.md` with a source link.
7. Record dependency or architecture decisions in `docs/DECISIONS.md`.
8. Do not commit `.env`, secrets, private keys, or personal data.
9. Update `docs/ROADMAP.md` if you complete or change a planned item.

## Code style

- Rust: `rustfmt` defaults, no `unwrap`/`expect`, explicit error enums, `require_auth` on every privileged function.
- TypeScript: strict mode, no `any` casts without a comment explaining why.
- No source file over 600 lines.

## Security issues

Report security vulnerabilities privately. See `SECURITY.md`.
