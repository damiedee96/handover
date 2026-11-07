/**
 * Typed, validated access to environment variables.
 * All values come from Vite's import.meta.env and are set at build time.
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

export type StellarNetwork = "TESTNET" | "MAINNET";

export const ENV = {
  network: (import.meta.env["VITE_STELLAR_NETWORK"] ?? "TESTNET") as StellarNetwork,
  rpcUrl: (import.meta.env["VITE_RPC_URL"] ??
    "https://soroban-testnet.stellar.org") as string,
  escrowContractId: (): string => required("VITE_ESCROW_CONTRACT_ID"),
  tokenContractId: (): string => required("VITE_TOKEN_CONTRACT_ID"),
} as const;

export const NETWORK_PASSPHRASE: Record<StellarNetwork, string> = {
  TESTNET: "Test SDF Network ; September 2015",
  MAINNET: "Public Global Stellar Network ; September 2015",
};
