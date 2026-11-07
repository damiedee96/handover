import { Link } from "react-router-dom";
import { WalletButton } from "./WalletButton";
import { ENV, NETWORK_PASSPHRASE } from "../lib/env";

// Derive a short network label from the passphrase for display.
const NETWORK_LABELS: Record<keyof typeof NETWORK_PASSPHRASE, string> = {
  TESTNET: "Testnet",
  MAINNET: "Mainnet",
};

export function NavBar() {
  return (
    <nav className="navbar">
      <Link to="/" className="navbar__brand">
        Handover
      </Link>
      <span className="navbar__network">{NETWORK_LABELS[ENV.network]}</span>
      <div className="navbar__links">
        <Link to="/send">Send</Link>
        <Link to="/receive">Receive</Link>
        <Link to="/status">Status</Link>
      </div>
      <WalletButton />
    </nav>
  );
}
