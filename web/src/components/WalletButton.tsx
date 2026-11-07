import { useWallet } from "../lib/walletContext";

export function WalletButton() {
  const { walletState, connect, disconnect } = useWallet();

  if (walletState.status === "connected") {
    const short =
      walletState.address.slice(0, 4) + "…" + walletState.address.slice(-4);
    return (
      <button onClick={disconnect} className="wallet-btn wallet-btn--connected">
        {short} ✕
      </button>
    );
  }

  if (walletState.status === "connecting") {
    return (
      <button disabled className="wallet-btn">
        Connecting…
      </button>
    );
  }

  if (walletState.status === "error") {
    return (
      <span className="wallet-error" title={walletState.message}>
        ⚠ Wallet error
      </span>
    );
  }

  return (
    <button onClick={connect} className="wallet-btn">
      Connect Freighter
    </button>
  );
}
