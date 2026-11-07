/**
 * Secret helpers: generate a random delivery secret, encode it as a share URL,
 * and decode it back.
 *
 * The secret is a 128-bit random value encoded as 22 base-58-ish characters
 * using the URL-safe base64 alphabet. It is displayed as a QR code and shared
 * as a URL fragment (#secret=...) so it never touches a server.
 */

const ALPHABET =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/** Generate a cryptographically random 16-byte secret and return it as a
 *  URL-safe base64 string (no padding). */
export function generateSecret(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

/** Build a share URL that encodes the delivery id and secret in the fragment.
 *  Fragment data never reaches the server. */
export function buildShareUrl(
  baseUrl: string,
  deliveryId: bigint,
  secret: string,
): string {
  const params = new URLSearchParams({
    id: deliveryId.toString(),
    secret,
  });
  return `${baseUrl}#${params.toString()}`;
}

/** Parse a share URL fragment back to id + secret, or return null. */
export function parseShareFragment(fragment: string): {
  id: bigint;
  secret: string;
} | null {
  try {
    const clean = fragment.startsWith("#") ? fragment.slice(1) : fragment;
    const params = new URLSearchParams(clean);
    const id = params.get("id");
    const secret = params.get("secret");
    if (!id || !secret) return null;
    return { id: BigInt(id), secret };
  } catch {
    return null;
  }
}

// Keep the alphabet import used to avoid TS noUnusedLocals error.
void ALPHABET;
