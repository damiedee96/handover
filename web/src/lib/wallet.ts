/**
 * Thin wrapper around the Freighter browser-extension API.
 *
 * Documentation: https://docs.freighter.app/docs/guide/usingFreighterBrowser
 *
 * The app never touches secret keys. All signing happens inside the extension.
 */
import {
  isConnected,
  requestAccess,
  getAddress,
  signTransaction,
} from "@stellar/freighter-api";
import { ENV, NETWORK_PASSPHRASE } from "./env";

export type WalletState =
  | { status: "disconnected" }
  | { status: "connecting" }
  | { status: "connected"; address: string }
  | { status: "error"; message: string };

/** Returns true if the Freighter extension is installed in this browser. */
export async function freighterInstalled(): Promise<boolean> {
  try {
    const result = await isConnected();
    return result.isConnected;
  } catch {
    return false;
  }
}

/** Ask Freighter for the user's public key. Returns the address string. */
export async function connectWallet(): Promise<string> {
  const accessResult = await requestAccess();
  if (accessResult.error) {
    throw new Error(accessResult.error);
  }

  const addrResult = await getAddress();
  if (addrResult.error) {
    throw new Error(addrResult.error);
  }
  return addrResult.address;
}

/** Sign an XDR-encoded transaction envelope and return the signed XDR. */
export async function signTx(
  unsignedXdr: string,
  signerAddress: string,
): Promise<string> {
  const result = await signTransaction(unsignedXdr, {
    networkPassphrase: NETWORK_PASSPHRASE[ENV.network],
    address: signerAddress,
  });
  if (result.error) {
    throw new Error(result.error);
  }
  return result.signedTxXdr;
}
