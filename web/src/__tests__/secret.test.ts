/**
 * Tests for the share-link helpers in src/lib/secret.ts
 *
 * Covers:
 *  - buildShareUrl puts the id in the query string
 *  - buildShareUrl does not put a secret in the URL (there is none)
 *  - parseDeliveryIdFromQuery round-trips the id
 *  - parseDeliveryIdFromQuery returns null for missing/bad input
 */
import { describe, it, expect } from "vitest";
import { buildShareUrl, parseDeliveryIdFromQuery } from "../lib/secret";

describe("buildShareUrl", () => {
  it("includes the delivery id in the query string", () => {
    const url = buildShareUrl("https://example.com/recipient", 42n);
    expect(url).toContain("id=42");
  });

  it("uses a query string, not a fragment", () => {
    const url = buildShareUrl("https://example.com/recipient", 1n);
    expect(url).toContain("?");
    expect(url).not.toContain("#");
  });

  it("does not contain a 'secret' parameter", () => {
    const url = buildShareUrl("https://example.com/recipient", 7n);
    expect(url).not.toContain("secret");
  });

  it("works for the courier lookup URL too", () => {
    const url = buildShareUrl("https://example.com/receive", 99n);
    expect(url).toContain("id=99");
  });
});

describe("parseDeliveryIdFromQuery", () => {
  it("parses a valid id from a query string", () => {
    expect(parseDeliveryIdFromQuery("?id=99")).toBe(99n);
  });

  it("parses without the leading ?", () => {
    expect(parseDeliveryIdFromQuery("id=7")).toBe(7n);
  });

  it("returns null when id is missing", () => {
    expect(parseDeliveryIdFromQuery("?foo=bar")).toBeNull();
    expect(parseDeliveryIdFromQuery("")).toBeNull();
  });

  it("round-trips a value from buildShareUrl", () => {
    const url = buildShareUrl("https://example.com/recipient", 123n);
    const search = new URL(url).search;
    expect(parseDeliveryIdFromQuery(search)).toBe(123n);
  });
});
