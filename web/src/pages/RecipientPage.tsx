/**
 * Recipient page.
 *
 * The recipient connects their Freighter wallet and calls confirm_receipt,
 * which releases the remainder of the escrow to the courier.
 *
 * Security: confirm_receipt requires the recipient's on-chain signature.
 * The courier cannot trigger this call — that is the fix for CVE-2026-001.
 */
import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import { useWallet } from "../lib/walletContext";
import { confirmReceipt, getDelivery, type Delivery } from "../lib/escrow";
import { parseDeliveryIdFromQuery } from "../lib/secret";
import { StatusBadge } from "../components/StatusBadge";

type PageState =
  | { phase: "lookup" }
  | { phase: "loading" }
  | { phase: "loaded"; delivery: Delivery }
  | { phase: "submitting"; delivery: Delivery }
  | { phase: "done" }
  | { phase: "error"; message: string };

export function RecipientPage() {
  const { walletState } = useWallet();
  const location = useLocation();

  const [idInput, setIdInput] = useState("");
  const [state, setState] = useState<PageState>({ phase: "lookup" });

  const connected =
    walletState.status === "connected" ? walletState.address : null;

  useEffect(() => {
    const id = parseDeliveryIdFromQuery(location.search);
    if (id !== null) {
      setIdInput(id.toString());
      loadDelivery(id);
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

  async function handleConfirmReceipt() {
    if (!connected || state.phase !== "loaded") return;
    const { delivery } = state;
    setState({ phase: "submitting", delivery });
    try {
      await confirmReceipt(connected, delivery.id);
      setState({ phase: "done" });
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
        <h1>Receipt confirmed ✓</h1>
        <p>
          The remainder of the escrow has been released to the courier. Thank
          you for confirming delivery.
        </p>
        <button
          type="button"
          className="btn-secondary"
          onClick={() => setState({ phase: "lookup" })}
        >
          Check another
        </button>
      </main>
    );
  }

  return (
    <main className="page">
      <h1>Confirm receipt</h1>

      {!connected && (
        <p className="alert alert--warn">
          Connect your Freighter wallet to confirm receipt of the parcel.
        </p>
      )}

      {state.phase === "error" && (
        <p className="alert alert--error" role="alert">
          {(state as { phase: "error"; message: string }).message}
        </p>
      )}

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

      {(state.phase === "loaded" || state.phase === "submitting") && (
        <section className="delivery-detail" aria-label="Delivery details">
          <h2>Delivery #{state.delivery.id.toString()}</h2>
          <dl className="detail-grid">
            <dt>Status</dt>
            <dd>
              <StatusBadge status={state.delivery.status} />
            </dd>
            <dt>Amount</dt>
            <dd>{(Number(state.delivery.amount) / 1e7).toFixed(7)}</dd>
            <dt>Pickup release</dt>
            <dd>{(state.delivery.pickupBps / 100).toFixed(2)}%</dd>
            <dt>Expiry ledger</dt>
            <dd>{state.delivery.expiryLedger}</dd>
          </dl>

          {state.delivery.status === "PickedUp" && (
            <>
              <p className="hint" aria-live="polite">
                The courier has confirmed pickup. Once you have received the
                parcel, confirm receipt below. Your wallet signature releases
                the remainder to the courier.
              </p>
              <button
                type="button"
                onClick={handleConfirmReceipt}
                disabled={!connected || state.phase === "submitting"}
                className="btn-primary"
              >
                {state.phase === "submitting"
                  ? "Confirming…"
                  : "Confirm receipt"}
              </button>
            </>
          )}

          {state.delivery.status === "Created" && (
            <p className="hint">
              Waiting for the courier to confirm pickup first.
            </p>
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
