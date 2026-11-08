/**
 * Share-link helpers.
 *
 * The QR code / share link shown to the recipient carries only the delivery id.
 * The recipient uses it to open the recipient page, connect their wallet, and
 * call confirm_receipt.  No secret is involved; wallet auth is the proof.
 */

/**
 * Build a share URL for the recipient that contains the delivery id.
 * Also used for the courier lookup link.
 */
export function buildShareUrl(baseUrl: string, deliveryId: bigint): string {
  const params = new URLSearchParams({ id: deliveryId.toString() });
  return `${baseUrl}?${params.toString()}`;
}

/**
 * Parse a delivery id from a URL query string, or return null.
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
