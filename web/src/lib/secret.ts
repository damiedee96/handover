/**
 * Secret helpers.
 *
 * Security model:
 *  - The SECRET belongs to the RECIPIENT (the person receiving the parcel).
 *  - The recipient gets the secret from the sender (via QR / share link) and
 *    reveals it to the courier only at the moment of physical handoff.
 *  - The courier page NEVER reads the secret from a URL. The courier types it
 *    manually after the recipient shows it to them.
 *  - The share URL contains ONLY the delivery id (so the courier can look up
 *    the delivery). It never encodes the secret.
 *
 * The secret itself is a 128-bit random value URL-safe base64 encoded.
 * Its SHA-256 hash is what goes on-chain; the plaintext never does.
 */

/** Generate a cryptographically random 128-bit secret as URL-safe base64 (no padding). */
export function generateSecret(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

/**
 * Build a courier look-up URL that contains ONLY the delivery id.
 * The secret is NOT included; the recipient shares it separately at handoff.
 */
export function buildCourierLookupUrl(
  baseUrl: string,
  deliveryId: bigint,
): string {
  const params = new URLSearchParams({ id: deliveryId.toString() });
  return `${baseUrl}?${params.toString()}`;
}

/**
 * Parse a delivery id from a courier URL query string, or return null.
 * The secret is intentionally absent from this URL.
 */
export function parseDeliveryIdFromQuery(search: string): bigint | null {
  try {
    const params = new URLSearchParams(
      search.startsWith("?") ? search.slice(1) : search,
    );
    const id = params.get("id");
    if (!id) return null;
    return BigInt(id);
  } catch {
    return null;
  }
}
