/**
 * Tests that the courier page NEVER pre-fills the secret from the URL.
 *
 * The share URL for the courier contains only the delivery id.
 * The secret input must never appear before the user manually triggers it.
 */
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { CourierPage } from "../pages/CourierPage";

// Mock wallet context so we don't need a real Freighter extension.
vi.mock("../lib/walletContext", () => ({
  useWallet: () => ({ walletState: { status: "disconnected" } }),
}));

// Mock escrow functions so no RPC call is made.
vi.mock("../lib/escrow", () => ({
  getDelivery: vi.fn(),
  confirmPickup: vi.fn(),
  completeDelivery: vi.fn(),
}));

function renderWithRoute(search: string) {
  return render(
    <MemoryRouter initialEntries={[`/receive${search}`]}>
      <Routes>
        <Route path="/receive" element={<CourierPage />} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("CourierPage secret pre-fill guard", () => {
  it("secret input is absent in the lookup phase (no URL)", () => {
    renderWithRoute("");
    // Secret input only appears after a delivery reaches PickedUp state.
    expect(screen.queryByLabelText(/secret/i)).toBeNull();
  });

  it("secret input is absent even when URL contains a legacy secret fragment", () => {
    // Old-style URL with a secret in the hash — must be ignored.
    renderWithRoute("?id=5#secret=shouldNotAppear");
    expect(screen.queryByLabelText(/secret/i)).toBeNull();
  });

  it("secret input is absent when only a delivery id is present in URL", () => {
    renderWithRoute("?id=42");
    // The courier page is in loading phase; no secret input is shown.
    expect(screen.queryByLabelText(/secret/i)).toBeNull();
  });
});
