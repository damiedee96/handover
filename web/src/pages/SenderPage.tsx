/**
 * Sender page – create a new delivery.
 *
 * The sender specifies the courier and the recipient.  No secret is generated
 * here.  Completion is authorised by the recipient's wallet signature, not by
 * a bearer secret.
 *
 * After creation the sender receives:
 *   • A QR code / link for the recipient → /recipient?id=<n>
 *   • A lookup link for the courier      → /receive?id=<n>
 */
import { useState } from "react";
import { QRCodeSVG } from "qrcode.react";
import { useWallet } from "../lib/walletContext";
import { createDelivery, currentLedger } from "../lib/escrow";
import { buildShareUrl } from "../lib/secret";
import { ENV } from "../lib/env";

type PageState =
  | { phase: "form" }
  | { phase: "submitting" }
  | {
      phase: "done";
      deliveryId: bigint;
      recipientUrl: string;
      courierUrl: string;
    }
  | { phase: "error"; message: string };

const LEDGERS_PER_DAY = 17_280;

export function SenderPage() {
  const { walletState } = useWallet();
  const [courier, setCourier] = useState("");
  const [recipient, setRecipient] = useState("");
  const [amount, setAmount] = useState("");
  const [pickupPct, setPickupPct] = useState("20");
  const [expiryDays, setExpiryDays] = useState("7");
  const [state, setState] = useState<PageState>({ phase: "form" });

  const connected =
    walletState.status === "connected" ? walletState.address : null;

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!connected) return;

    const amountNum = parseFloat(amount);
    if (!amountNum || amountNum <= 0) {
      setState({ phase: "error", message: "Amount must be positive." });
      return;
    }
    const pickupBps = Math.round(parseFloat(pickupPct) * 100);
    if (pickupBps < 0 || pickupBps > 5000) {
      setState({ phase: "error", message: "Pickup % must be 0–50." });
      return;
    }

    setState({ phase: "submitting" });
    try {
      const ledger = await currentLedger();
      const expiryLedger =
        ledger + Math.round(parseFloat(expiryDays) * LEDGERS_PER_DAY);
      const amountStroops = BigInt(Math.round(amountNum * 1e7));

      const deliveryId = await createDelivery({
        sender: connected,
        courier,
        recipient,
        amount: amountStroops,
        pickupBps,
        expiryLedger,
      });

      const origin = window.location.origin;
      const recipientUrl = buildShareUrl(`${origin}/recipient`, deliveryId);
      const courierUrl = buildShareUrl(`${origin}/receive`, deliveryId);

      setState({ phase: "done", deliveryId, recipientUrl, courierUrl });
    } catch (err) {
      setState({
        phase: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }

  if (state.phase === "done") {
    return (
      <main className="page">
        <h1>Delivery created ✓</h1>
        <p>
          Delivery <strong>#{state.deliveryId.toString()}</strong> is funded on
          Testnet.
        </p>

        {/* Recipient link – they use wallet auth to confirm receipt */}
        <section className="secret-section" aria-label="Recipient link">
          <h2>Send to the recipient</h2>
          <p className="hint">
            The recipient opens this link, connects their Freighter wallet, and
            confirms receipt after the parcel arrives. Their wallet signature
            releases the remainder to the courier.
          </p>
          <div className="qr-wrapper">
            <QRCodeSVG
              value={state.recipientUrl}
              size={220}
              aria-label="QR code for the recipient confirm-receipt page"
            />
          </div>
          <div className="share-link-row">
            <input
              readOnly
              value={state.recipientUrl}
              aria-label="Recipient link"
              className="share-link-input"
              onFocus={(e) => e.currentTarget.select()}
            />
            <button
              type="button"
              onClick={() =>
                navigator.clipboard.writeText(state.recipientUrl)
              }
            >
              Copy
            </button>
          </div>
        </section>

        {/* Courier lookup link */}
        <section className="secret-section" aria-label="Courier lookup link">
          <h2>Send this link to the courier</h2>
          <p className="hint">
            The courier uses this to confirm pickup.
          </p>
          <div className="share-link-row">
            <input
              readOnly
              value={state.courierUrl}
              aria-label="Courier lookup link"
              className="share-link-input"
              onFocus={(e) => e.currentTarget.select()}
            />
            <button
              type="button"
              onClick={() => navigator.clipboard.writeText(state.courierUrl)}
            >
              Copy
            </button>
          </div>
        </section>

        <button
          type="button"
          className="btn-secondary"
          onClick={() => setState({ phase: "form" })}
        >
          Create another
        </button>
      </main>
    );
  }

  return (
    <main className="page">
      <h1>Create a delivery</h1>

      {!connected && (
        <p className="alert alert--warn">
          Connect your Freighter wallet to continue.
        </p>
      )}

      {state.phase === "error" && (
        <p className="alert alert--error" role="alert">
          {state.message}
        </p>
      )}

      <form onSubmit={handleSubmit} className="delivery-form">
        <label htmlFor="courier">Courier address (G…)</label>
        <input
          id="courier"
          type="text"
          required
          placeholder="GABC…"
          value={courier}
          onChange={(e) => setCourier(e.target.value)}
          pattern="G[A-Z2-7]{55}"
          title="Stellar public key starting with G"
          aria-describedby="courier-hint"
        />
        <span id="courier-hint" className="field-hint">
          The Stellar address of the courier who will pick up the delivery.
        </span>

        <label htmlFor="recipient">Recipient address (G…)</label>
        <input
          id="recipient"
          type="text"
          required
          placeholder="GXYZ…"
          value={recipient}
          onChange={(e) => setRecipient(e.target.value)}
          pattern="G[A-Z2-7]{55}"
          title="Stellar public key starting with G"
          aria-describedby="recipient-hint"
        />
        <span id="recipient-hint" className="field-hint">
          The Stellar address of the person receiving the parcel. They will
          confirm receipt with their wallet.
        </span>

        <label htmlFor="token-display">Token (Testnet)</label>
        <input
          id="token-display"
          type="text"
          readOnly
          value={ENV.tokenContractId()}
          aria-label="Token contract address (read-only, set by env)"
        />

        <label htmlFor="amount">Amount (token units)</label>
        <input
          id="amount"
          type="number"
          required
          min="0.0000001"
          step="any"
          placeholder="100.00"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
        />

        <label htmlFor="pickup-pct">Pickup release (%)</label>
        <input
          id="pickup-pct"
          type="number"
          required
          min="0"
          max="50"
          step="0.01"
          placeholder="20"
          value={pickupPct}
          onChange={(e) => setPickupPct(e.target.value)}
          aria-describedby="pickup-hint"
        />
        <span id="pickup-hint" className="field-hint">
          0–50% paid to the courier on pickup; the rest on confirmed receipt.
        </span>

        <label htmlFor="expiry-days">Expiry (days)</label>
        <input
          id="expiry-days"
          type="number"
          required
          min="1"
          step="1"
          placeholder="7"
          value={expiryDays}
          onChange={(e) => setExpiryDays(e.target.value)}
        />

        <button
          type="submit"
          disabled={!connected || state.phase === "submitting"}
          className="btn-primary"
        >
          {state.phase === "submitting" ? "Submitting…" : "Create delivery"}
        </button>
      </form>
    </main>
  );
}
