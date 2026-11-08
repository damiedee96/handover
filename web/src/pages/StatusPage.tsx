/**
 * Status page – look up any delivery by id and optionally refund it.
 */
import { useState } from "react";
import { useWallet } from "../lib/walletContext";
import { getDelivery, refund, type Delivery } from "../lib/escrow";
import { StatusBadge } from "../components/StatusBadge";

type PageState =
  | { phase: "idle" }
  | { phase: "loading" }
  | { phase: "loaded"; delivery: Delivery }
  | { phase: "refunding"; delivery: Delivery }
  | { phase: "refunded" }
  | { phase: "error"; message: string };

export function StatusPage() {
  const { walletState } = useWallet();
  const [idInput, setIdInput] = useState("");
  const [state, setState] = useState<PageState>({ phase: "idle" });

  const connected =
    walletState.status === "connected" ? walletState.address : null;

  async function handleLookup(e: React.FormEvent) {
    e.preventDefault();
    setState({ phase: "loading" });
    try {
      const delivery = await getDelivery(BigInt(idInput));
      setState({ phase: "loaded", delivery });
    } catch (err) {
      setState({
        phase: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }

  async function handleRefund() {
    if (!connected || state.phase !== "loaded") return;
    const { delivery } = state;
    setState({ phase: "refunding", delivery });
    try {
      await refund(connected, delivery.id);
      setState({ phase: "refunded" });
    } catch (err) {
      setState({
        phase: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }

  if (state.phase === "refunded") {
    return (
      <main className="page">
        <h1>Refund sent ✓</h1>
        <p>The unreleased amount has been returned to the sender.</p>
        <button
          type="button"
          className="btn-secondary"
          onClick={() => setState({ phase: "idle" })}
        >
          Look up another
        </button>
      </main>
    );
  }

  return (
    <main className="page">
      <h1>Delivery status</h1>

      {state.phase === "error" && (
        <p className="alert alert--error" role="alert">
          {(state as { phase: "error"; message: string }).message}
        </p>
      )}

      <form onSubmit={handleLookup} className="delivery-form">
        <label htmlFor="status-id">Delivery ID</label>
        <input
          id="status-id"
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
          {state.phase === "loading" ? "Loading…" : "Look up"}
        </button>
      </form>

      {(state.phase === "loaded" || state.phase === "refunding") && (
        <section className="delivery-detail" aria-label="Delivery details">
          <h2>Delivery #{state.delivery.id.toString()}</h2>

          <dl className="detail-grid">
            <dt>Status</dt>
            <dd><StatusBadge status={state.delivery.status} /></dd>
            <dt>Sender</dt>
            <dd className="addr">{state.delivery.sender}</dd>
            <dt>Courier</dt>
            <dd className="addr">{state.delivery.courier}</dd>
            <dt>Recipient</dt>
            <dd className="addr">{state.delivery.recipient}</dd>
            <dt>Amount</dt>
            <dd>{(Number(state.delivery.amount) / 1e7).toFixed(7)}</dd>
            <dt>Released</dt>
            <dd>{(Number(state.delivery.released) / 1e7).toFixed(7)}</dd>
            <dt>Pickup release</dt>
            <dd>{(state.delivery.pickupBps / 100).toFixed(2)}%</dd>
            <dt>Expiry ledger</dt>
            <dd>{state.delivery.expiryLedger}</dd>
          </dl>

          {(state.delivery.status === "Created" ||
            state.delivery.status === "PickedUp") && (
            <button
              type="button"
              onClick={handleRefund}
              disabled={!connected || state.phase === "refunding"}
              className="btn-secondary"
              aria-describedby="refund-hint"
            >
              {state.phase === "refunding" ? "Processing…" : "Refund"}
            </button>
          )}
          <span id="refund-hint" className="field-hint">
            Refund is only allowed after the expiry ledger has passed.
          </span>
        </section>
      )}
    </main>
  );
}
