/**
 * Courier page.
 *
 * Secret-flow model:
 *  - The courier arrives here via the lookup link which contains ONLY the
 *    delivery id (no secret).
 *  - The courier confirms pickup on-chain.
 *  - At physical handoff, the recipient shows the courier their secret code.
 *    The courier types it in manually.  It is NEVER pre-filled from a URL.
 *  - complete_delivery is called with the typed secret.
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
import { parseDeliveryIdFromQuery } from "../lib/secret";
import { StatusBadge } from "../components/StatusBadge";

type PageState =
  | { phase: "lookup" }
  | { phase: "loading" }
  | { phase: "loaded"; delivery: Delivery }
  | { phase: "submitting"; delivery: Delivery }
  | { phase: "done"; action: "pickup" | "complete" }
  | { phase: "error"; message: string };

export function CourierPage() {
  const { walletState } = useWallet();
  const location = useLocation();

  const [idInput, setIdInput] = useState("");
  // secretInput is ALWAYS empty on load; the courier types it manually.
  const [secretInput, setSecretInput] = useState("");
  const [state, setState] = useState<PageState>({ phase: "lookup" });

  const connected =
    walletState.status === "connected" ? walletState.address : null;

  // Read only the delivery id from the URL query string — never the secret.
  useEffect(() => {
    const parsed = parseDeliveryIdFromQuery(location.search);
    if (parsed !== null) {
      const idStr = parsed.toString();
      setIdInput(idStr);
      loadDelivery(parsed);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function loadDelivery(id: bigint) {
    setState({ phase: "loading" });
    try {
      const delivery = await getDelivery(id);
      setState({ phase: "loaded", delivery });
    } catch (err) {
      setState({
        phase: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }

  async function handleLookup(e: React.FormEvent) {
    e.preventDefault();
    await loadDelivery(BigInt(idInput));
  }

  async function handleConfirmPickup() {
    if (!connected || state.phase !== "loaded") return;
    const { delivery } = state;
    setState({ phase: "submitting", delivery });
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
    const { delivery } = state;
    setState({ phase: "submitting", delivery });
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
          {(state as { phase: "error"; message: string }).message}
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
                placeholder="Ask the recipient to show you their code"
                value={secretInput}
                onChange={(e) => setSecretInput(e.target.value)}
                aria-describedby="secret-hint"
                autoComplete="off"
              />
              <span id="secret-hint" className="field-hint">
                The recipient will show you this code at handoff. Type it here
                exactly as shown — do not scan a URL.
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
