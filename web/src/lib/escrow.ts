/**
 * Escrow contract client.
 *
 * Builds, simulates, and submits Soroban contract invocations using
 * @stellar/stellar-sdk v13.  Signing is delegated to the wallet module so no
 * secret key ever passes through this code.
 *
 * Stellar SDK docs: https://stellar.github.io/js-stellar-sdk/
 */
import {
  Contract,
  rpc,
  TransactionBuilder,
  BASE_FEE,
  xdr,
  nativeToScVal,
  scValToNative,
  Address,
  hash,
} from "@stellar/stellar-sdk";
import { ENV, NETWORK_PASSPHRASE, type StellarNetwork } from "./env";
import { signTx } from "./wallet";

// ─── Types ──────────────────────────────────────────────────────────────────

export type DeliveryStatus =
  | "Created"
  | "PickedUp"
  | "Completed"
  | "Refunded";

export interface Delivery {
  id: bigint;
  sender: string;
  courier: string;
  token: string;
  amount: bigint;
  secretHash: Uint8Array;
  pickupBps: number;
  expiryLedger: number;
  status: DeliveryStatus;
  released: bigint;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function networkPassphrase(): string {
  const map: Record<StellarNetwork, string> = NETWORK_PASSPHRASE;
  return map[ENV.network];
}

function rpcServer(): rpc.Server {
  return new rpc.Server(ENV.rpcUrl, { allowHttp: false });
}

/** SHA-256 hash of a UTF-8 string, returned as a Uint8Array. */
export function sha256Bytes(secret: string): Uint8Array {
  const bytes = new TextEncoder().encode(secret);
  // hash() from stellar-sdk accepts a Buffer-like; Uint8Array is compatible.
  return hash(bytes as unknown as Parameters<typeof hash>[0]);
}

/** Encode arbitrary bytes as an xdr ScVal Bytes value. */
function bytesToScVal(bytes: Uint8Array): xdr.ScVal {
  // xdr.ScVal.scvBytes is typed as Buffer but accepts Uint8Array at runtime.
  return xdr.ScVal.scvBytes(bytes as unknown as Parameters<typeof xdr.ScVal.scvBytes>[0]);
}

/** Parse the raw ScVal map returned by get_delivery into a Delivery object. */
function parseDelivery(id: bigint, raw: unknown): Delivery {
  const obj = raw as Record<string, unknown>;
  return {
    id,
    sender: obj["sender"] as string,
    courier: obj["courier"] as string,
    token: obj["token"] as string,
    amount: BigInt(obj["amount"] as string | number),
    secretHash: obj["secret_hash"] as Uint8Array,
    pickupBps: Number(obj["pickup_bps"]),
    expiryLedger: Number(obj["expiry_ledger"]),
    status: obj["status"] as DeliveryStatus,
    released: BigInt(obj["released"] as string | number),
  };
}

// ─── Contract invocation core ────────────────────────────────────────────────

/** Build, simulate, sign (via wallet), and submit a contract call. */
async function invoke(
  callerAddress: string,
  method: string,
  args: xdr.ScVal[],
): Promise<xdr.ScVal> {
  const server = rpcServer();
  const account = await server.getAccount(callerAddress);
  const contract = new Contract(ENV.escrowContractId());

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: networkPassphrase(),
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  // Simulate to get the correct resource fees and read/write footprint.
  const simResult = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation error: ${simResult.error}`);
  }

  const preparedTx = rpc.assembleTransaction(tx, simResult).build();

  // Sign via Freighter — no secret key involved.
  const signedXdr = await signTx(preparedTx.toXDR(), callerAddress);

  const submitResult = await server.sendTransaction(
    TransactionBuilder.fromXDR(signedXdr, networkPassphrase()),
  );

  if (submitResult.status === "ERROR") {
    throw new Error(`Submit error: ${JSON.stringify(submitResult.errorResult)}`);
  }

  // Poll for finality.
  const txHash = submitResult.hash;
  for (let i = 0; i < 30; i++) {
    await new Promise((r) => setTimeout(r, 2000));
    const status = await server.getTransaction(txHash);
    if (status.status === rpc.Api.GetTransactionStatus.SUCCESS) {
      return status.returnValue ?? xdr.ScVal.scvVoid();
    }
    if (status.status === rpc.Api.GetTransactionStatus.FAILED) {
      throw new Error("Transaction failed on-chain");
    }
  }
  throw new Error("Transaction polling timed out after 60 s");
}

// ─── Public contract functions ───────────────────────────────────────────────

/**
 * create_delivery – funded by sender.
 * Returns the numeric delivery id as a bigint.
 */
export async function createDelivery(params: {
  sender: string;
  courier: string;
  amount: bigint;
  secretHash: Uint8Array;
  pickupBps: number;
  expiryLedger: number;
}): Promise<bigint> {
  const { sender, courier, amount, secretHash, pickupBps, expiryLedger } =
    params;

  const args: xdr.ScVal[] = [
    new Address(sender).toScVal(),
    new Address(courier).toScVal(),
    new Address(ENV.tokenContractId()).toScVal(),
    nativeToScVal(amount, { type: "i128" }),
    bytesToScVal(secretHash),
    nativeToScVal(pickupBps, { type: "u32" }),
    nativeToScVal(expiryLedger, { type: "u32" }),
  ];

  const result = await invoke(sender, "create_delivery", args);
  return BigInt(scValToNative(result) as number | string);
}

/** confirm_pickup – must be called by the courier. */
export async function confirmPickup(
  courier: string,
  id: bigint,
): Promise<void> {
  await invoke(courier, "confirm_pickup", [nativeToScVal(id, { type: "u64" })]);
}

/** complete_delivery – must be called by the courier with the plaintext secret. */
export async function completeDelivery(
  courier: string,
  id: bigint,
  secret: string,
): Promise<void> {
  const secretBytes = new TextEncoder().encode(secret);
  await invoke(courier, "complete_delivery", [
    nativeToScVal(id, { type: "u64" }),
    bytesToScVal(secretBytes),
  ]);
}

/** refund – must be called by the sender after expiry. */
export async function refund(sender: string, id: bigint): Promise<void> {
  await invoke(sender, "refund", [nativeToScVal(id, { type: "u64" })]);
}

/** get_delivery – read-only simulation, no wallet required. */
export async function getDelivery(id: bigint): Promise<Delivery> {
  const server = rpcServer();
  const contract = new Contract(ENV.escrowContractId());

  // A well-known funded account used only to satisfy the builder's type.
  const HORIZON_FRIENDLY =
    "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN";

  const account = await server.getAccount(HORIZON_FRIENDLY);

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: networkPassphrase(),
  })
    .addOperation(
      contract.call("get_delivery", nativeToScVal(id, { type: "u64" })),
    )
    .setTimeout(30)
    .build();

  const simResult = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation error: ${simResult.error}`);
  }
  if (!("result" in simResult) || !simResult.result) {
    throw new Error("No result from simulation");
  }

  const native = scValToNative(simResult.result.retval);
  return parseDelivery(id, native);
}

/** Fetch the current ledger sequence number from the RPC. */
export async function currentLedger(): Promise<number> {
  const server = rpcServer();
  const info = await server.getLatestLedger();
  return info.sequence;
}

/** Human-readable network labels for display. */
export const NETWORKS: Record<StellarNetwork, string> = {
  TESTNET: "Testnet",
  MAINNET: "Mainnet",
};
