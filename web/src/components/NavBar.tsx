import { Link } from "react-router-dom";
import { WalletButton } from "./WalletButton";

export function NavBar() {
  return (
    <nav className="navbar">
      <Link to="/" className="navbar__brand">
        Handover
      </Link>
      <span className="navbar__network">Testnet</span>
      <div className="navbar__links">
        <Link to="/send">Send</Link>
        <Link to="/receive">Receive</Link>
        <Link to="/status">Status</Link>
      </div>
      <WalletButton />
    </nav>
  );
}
