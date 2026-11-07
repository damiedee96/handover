/**
 * Courier (receiver) page.
 *
 * The courier arrives here either via:
 *   • The share link (fragment contains id + secret)
 *   • Manually entering a delivery id
 *
 * Flow:
 *   1. Load delivery details.
 *   2. Confirm pickup (releases pickup_bps fraction).
 *   3. Enter the secret code → complete delivery (releases remainder).
 */
import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import { useWallet } from "../lib/walletContext";
import {
  confirmPickup,
  completeDelivery,
  getDelivery,
  type Delivery,
} from "../lib/escrow";
import { parseShareFragment } from "../lib/secret";
import { StatusBadge } from "../components/StatusBadge";

type PageState =
  | { phase: "lookup" }
  | { phase: "loading" }
  | { phase: "loaded"; delivery: Delivery; prefillSecret: string }
  | { phase: "submitting"; delivery: Delivery; prefillSecret: string }
  | { phase: "done"; action: "pickup" | "complete" }
  | { phase: "error"; message: string };

export function CourierPage() {
  const { walletState } = useWallet();
  const location = useLocation();

  const [idInput, setIdInput] = useState("");
  const [secretInput, setSecretInput] = useState("");
  const [state, setState] = useState<PageState>({ phase: "lookup" });

  const connected =
    walletState.status === "connected" ? walletState.address : null;

  // Parse fragment on mount.
  useEffect(() => {
    const parsed = parseShareFragment(location.hash);
    if (parsed) {
      setIdInput(parsed.id.toString());
      setSecretInput(parsed.secret);
      loadDelivery(parsed.id, parsed.secret);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function loadDelivery(id: bigint, prefillSecret = "") {
    setState({ phase: "loading" });
    try {
      const delivery = await getDelivery(id);
      setState({ phase: "loaded", delivery, prefillSecret });
    } catch (err) {
      setState({
        phase: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }

  async function handleLookup(e: React.FormEvent) {
    e.preventDefault();
    await loadDelivery(BigInt(idInput), secretInput);
  }

  async function handleConfirmPickup() {
    if (!connected || state.phase !== "loaded") return;
    const { delivery, prefillSecret } = state;
    setState({ phase: "submitting", delivery, prefillSecret });
    try {
      await confirmPickup(connected, delivery.id);
      setState({ phase: "done", action: "pickup" });
    } catch (err) {
      setState({
        phase: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }

  async function handleComplete(e: React.FormEvent) {
    e.preventDefault();
    if (!connected || state.phase !== "loaded") return;
    const { delivery, prefillSecret } = state;
    setState({ phase: "submitting", delivery, prefillSecret });
    try {
      await completeDelivery(connected, delivery.id, secretInput);
      setState({ phase: "done", action: "complete" });
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
        <h1>
          {state.action === "pickup"
            ? "Pickup confirmed ✓"
            : "Delivery complete ✓"}
        </h1>
        <p>
          {state.action === "pickup"
            ? "Your pickup payment has been released to your wallet."
            : "The full delivery amount has been released to your wallet."}
        </p>
        <button
          type="button"
          className="btn-secondary"
          onClick={() => setState({ phase: "lookup" })}
        >
          New delivery
        </button>
      </main>
    );
  }

  return (
    <main className="page">
      <h1>Courier delivery</h1>

      {!connected && (
        <p className="alert alert--warn">
          Connect your Freighter wallet to confirm pickup or complete delivery.
        </p>
      )}

      {state.phase === "error" && (
        <p className="alert alert--error" role="alert">
          {state.message}
        </p>
      )}

      {/* Lookup form */}
      {(state.phase === "lookup" || state.phase === "loading") && (
        <form onSubmit={handleLookup} className="delivery-form">
          <label htmlFor="delivery-id">Delivery ID</label>
          <input
            id="delivery-id"
            type="number"
            required
            min="0"
            placeholder="42"
            value={idInput}
            onChange={(e) => setIdInput(e.target.value)}
          />

          <label htmlFor="secret-lookup">Secret code (optional for lookup)</label>
          <input
            id="secret-lookup"
            type="text"
            placeholder="Paste from share link"
            value={secretInput}
            onChange={(e) => setSecretInput(e.target.value)}
          />

          <button
            type="submit"
            disabled={state.phase === "loading"}
            className="btn-primary"
          >
            {state.phase === "loading" ? "Loading…" : "Look up delivery"}
          </button>
        </form>
      )}

      {/* Delivery detail + actions */}
      {(state.phase === "loaded" || state.phase === "submitting") && (
        <section className="delivery-detail" aria-label="Delivery details">
          <h2>Delivery #{state.delivery.id.toString()}</h2>
          <dl className="detail-grid">
            <dt>Status</dt>
            <dd><StatusBadge status={state.delivery.status} /></dd>
            <dt>Amount</dt>
            <dd>{(Number(state.delivery.amount) / 1e7).toFixed(7)}</dd>
            <dt>Pickup release</dt>
            <dd>{(state.delivery.pickupBps / 100).toFixed(2)}%</dd>
            <dt>Expiry ledger</dt>
            <dd>{state.delivery.expiryLedger}</dd>
          </dl>

          {state.delivery.status === "Created" && (
            <button
              type="button"
              onClick={handleConfirmPickup}
              disabled={!connected || state.phase === "submitting"}
              className="btn-primary"
            >
              {state.phase === "submitting" ? "Confirming…" : "Confirm pickup"}
            </button>
          )}

          {state.delivery.status === "PickedUp" && (
            <form onSubmit={handleComplete} className="delivery-form">
              <label htmlFor="secret-complete">Secret code</label>
              <input
                id="secret-complete"
                type="text"
                required
                placeholder="Paste the secret from the sender"
                value={secretInput}
                onChange={(e) => setSecretInput(e.target.value)}
                aria-describedby="secret-hint"
              />
              <span id="secret-hint" className="field-hint">
                The sender shared this code with you — scan their QR or paste from
                the link.
              </span>
              <button
                type="submit"
                disabled={!connected || state.phase === "submitting"}
                className="btn-primary"
              >
                {state.phase === "submitting"
                  ? "Completing…"
                  : "Complete delivery"}
              </button>
            </form>
          )}

          {(state.delivery.status === "Completed" ||
            state.delivery.status === "Refunded") && (
            <p className="hint">This delivery is already finalised.</p>
          )}
        </section>
      )}
    </main>
  );
}
