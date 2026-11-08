/**
 * Tests for the secret helpers in src/lib/secret.ts
 *
 * Covers:
 *  - generateSecret returns a non-empty URL-safe base64 string
 *  - Two calls return different values (no reuse)
 *  - buildCourierLookupUrl puts the id in the query and NEVER the secret
 *  - parseDeliveryIdFromQuery round-trips the id
 *  - parseDeliveryIdFromQuery returns null for missing/bad input
 */
import { describe, it, expect } from "vitest";
import {
  generateSecret,
  buildCourierLookupUrl,
  parseDeliveryIdFromQuery,
} from "../lib/secret";

describe("generateSecret", () => {
  it("returns a non-empty string", () => {
    expect(generateSecret().length).toBeGreaterThan(0);
  });

  it("uses only URL-safe base64 characters (no +, /, or =)", () => {
    const secret = generateSecret();
    expect(secret).not.toMatch(/[+/=]/);
  });

  it("returns different values on successive calls", () => {
    expect(generateSecret()).not.toBe(generateSecret());
  });
});

describe("buildCourierLookupUrl", () => {
  it("includes the delivery id in the query string", () => {
    const url = buildCourierLookupUrl("https://example.com/receive", 42n);
    expect(url).toContain("id=42");
  });

  it("does NOT include a secret parameter", () => {
    const secret = generateSecret();
    const url = buildCourierLookupUrl("https://example.com/receive", 7n);
    // Verify the url does not contain the secret value and has no 'secret' key.
    expect(url).not.toContain("secret");
    expect(url).not.toContain(secret);
  });

  it("uses a query string, not a fragment", () => {
    const url = buildCourierLookupUrl("https://example.com/receive", 1n);
    expect(url).toContain("?");
    expect(url).not.toContain("#");
  });
});

describe("parseDeliveryIdFromQuery", () => {
  it("parses a valid id from a query string", () => {
    const result = parseDeliveryIdFromQuery("?id=99");
    expect(result).toBe(99n);
  });

  it("parses without the leading ?", () => {
    const result = parseDeliveryIdFromQuery("id=7");
    expect(result).toBe(7n);
  });

  it("returns null when id is missing", () => {
    expect(parseDeliveryIdFromQuery("?foo=bar")).toBeNull();
    expect(parseDeliveryIdFromQuery("")).toBeNull();
  });

  it("round-trips a value from buildCourierLookupUrl", () => {
    const url = buildCourierLookupUrl("https://example.com/receive", 123n);
    const search = new URL(url).search;
    expect(parseDeliveryIdFromQuery(search)).toBe(123n);
  });
});
