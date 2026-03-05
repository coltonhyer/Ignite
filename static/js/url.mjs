/**
 * Builds a relative share URL containing the secret ID and base64url-encoded decryption key.
 *
 * @param {string} id - The UUID or identifier of the secret.
 * @param {string} base64urlKey - The pre-encoded base64url decryption key.
 * @returns {string} The relative URL in the format `/s/{id}#{base64urlKey}`.
 */
export function buildShareUrl(id, base64urlKey) {
  return `/s/${id}#${base64urlKey}`;
}

/**
 * Parses a share URL to extract the secret ID and base64url-encoded decryption key.
 *
 * If no urlString is provided, it attempts to read from the global `window.location`.
 *
 * @param {string} [urlString] - Optional URL string to parse instead of `window.location`.
 * @returns {{ id: string, key: string } | null} The extracted ID and key, or null if invalid.
 */
export function parseShareUrl(urlString) {
  let pathname = '';
  let hash = '';

  if (urlString) {
    try {
      // If it's just a path like /s/123#key, we need a base to parse it with URL
      const base = 'http://localhost';
      const parsedUrl = new URL(urlString, base);
      pathname = parsedUrl.pathname;
      hash = parsedUrl.hash;
    } catch (e) {
      return null;
    }
  } else if (typeof window !== 'undefined' && window.location) {
    pathname = window.location.pathname;
    hash = window.location.hash;
  } else {
    return null; // No context to parse from
  }

  // The pathname should be /s/:id
  const pathMatch = pathname.match(/^\/s\/([^\/]+)$/);
  if (!pathMatch) {
    return null;
  }

  const id = pathMatch[1];

  // The hash should start with # and have content
  if (!hash || hash.length <= 1 || !hash.startsWith('#')) {
    return null;
  }

  const key = hash.substring(1); // Remove the '#'

  return { id, key };
}
