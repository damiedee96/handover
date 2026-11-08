/**
 * Tests that the courier page has no secret input at all.
 *
 * With the new design, completion is triggered by the RECIPIENT's wallet
 * signature on the recipient page.  The courier page only has a confirm-pickup
 * button; there is no secret field anywhere in its UI.
 */
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { CourierPage } from "../pages/CourierPage";

vi.mock("../lib/walletContext", () => ({
  useWallet: () => ({ walletState: { status: "disconnected" } }),
}));

vi.mock("../lib/escrow", () => ({
  getDelivery: vi.fn(),
  confirmPickup: vi.fn(),
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

describe("CourierPage — no secret input", () => {
  it("has no secret input when no query string is present", () => {
    renderWithRoute("");
    expect(screen.queryByLabelText(/secret/i)).toBeNull();
  });

  it("has no secret input when a delivery id is in the URL", () => {
    renderWithRoute("?id=42");
    expect(screen.queryByLabelText(/secret/i)).toBeNull();
  });

  it("has no secret input even if a legacy hash fragment is present", () => {
    renderWithRoute("?id=5#secret=shouldNotAppear");
    expect(screen.queryByLabelText(/secret/i)).toBeNull();
  });
});
