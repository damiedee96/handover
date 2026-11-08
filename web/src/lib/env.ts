/**
 * Typed, validated access to environment variables.
 *
 * This app only supports Stellar Testnet.  If a non-TESTNET value is set the
 * app throws at startup rather than silently operating on Mainnet.
 */

function required(key: string): string {
  const val = import.meta.env[key] as string | undefined;
  if (!val) {
    throw new Error(
      `Missing environment variable ${key}. Copy .env.example to .env and fill in values.`,
    );
  }
  return val;
}

// Enforce TESTNET-only at module load time.
const rawNetwork = (import.meta.env["VITE_STELLAR_NETWORK"] as string | undefined) ?? "TESTNET";
if (rawNetwork !== "TESTNET") {
  throw new Error(
    `This app only supports TESTNET. Set VITE_STELLAR_NETWORK=TESTNET (got "${rawNetwork}").`,
  );
}

export const ENV = {
  network: "TESTNET" as const,
  rpcUrl: (import.meta.env["VITE_RPC_URL"] as string | undefined) ??
    "https://soroban-testnet.stellar.org",
  escrowContractId: (): string => required("VITE_ESCROW_CONTRACT_ID"),
  tokenContractId: (): string => required("VITE_TOKEN_CONTRACT_ID"),
} as const;

export const NETWORK_PASSPHRASE = "Test SDF Network ; September 2015";
