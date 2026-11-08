import { Link } from "react-router-dom";

export function HomePage() {
  return (
    <main className="page home">
      <h1>Handover</h1>
      <p className="tagline">
        On-chain escrow for in-person deliveries on Stellar Testnet.
      </p>

      <ul className="home-cards" aria-label="Available actions">
        <li>
          <Link to="/send" className="home-card">
            <span className="home-card__icon" aria-hidden="true">
              📦
            </span>
            <strong>Send</strong>
            <span>Create a delivery and fund the escrow</span>
          </Link>
        </li>
        <li>
          <Link to="/receive" className="home-card">
            <span className="home-card__icon" aria-hidden="true">
              🛵
            </span>
            <strong>Courier</strong>
            <span>Confirm pickup of a delivery</span>
          </Link>
        </li>
        <li>
          <Link to="/recipient" className="home-card">
            <span className="home-card__icon" aria-hidden="true">
              ✅
            </span>
            <strong>Recipient</strong>
            <span>Confirm receipt to release payment to the courier</span>
          </Link>
        </li>
        <li>
          <Link to="/status" className="home-card">
            <span className="home-card__icon" aria-hidden="true">
              🔍
            </span>
            <strong>Status</strong>
            <span>Check any delivery or request a refund</span>
          </Link>
        </li>
      </ul>
    </main>
  );
}
