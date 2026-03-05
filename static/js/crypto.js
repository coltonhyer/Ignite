// AES-256-GCM Encryption and Decryption Module
// Implements client-side encryption using the Web Crypto API.

// Helper to encode a standard Uint8Array or ArrayBuffer to base64 string
function bufferToBase64(buffer) {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
}

// Helper to decode a standard base64 string to Uint8Array
function base64ToBuffer(base64) {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
}

// Helper to encode a buffer to base64url string (URL-safe, no padding)
function bufferToBase64Url(buffer) {
    let b64 = bufferToBase64(buffer);
    return b64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

// Helper to decode a base64url string to Uint8Array
function base64UrlToBuffer(b64Url) {
    let b64 = b64Url.replace(/-/g, '+').replace(/_/g, '/');
    while (b64.length % 4) {
        b64 += '=';
    }
    return base64ToBuffer(b64);
}

/**
 * Encrypts a plaintext string using AES-256-GCM.
 * @param {string} plaintext - The UTF-8 string to encrypt.
 * @returns {Promise<{ciphertext: string, nonce: string, key: CryptoKey}>}
 */
export async function encrypt(plaintext) {
    const key = await crypto.subtle.generateKey(
        {
            name: "AES-GCM",
            length: 256,
        },
        true, // extractable
        ["encrypt", "decrypt"]
    );

    const nonce = crypto.getRandomValues(new Uint8Array(12));
    const encoder = new TextEncoder();
    const encodedPlaintext = encoder.encode(plaintext);

    const ciphertextBuffer = await crypto.subtle.encrypt(
        {
            name: "AES-GCM",
            iv: nonce
        },
        key,
        encodedPlaintext
    );

    return {
        ciphertext: bufferToBase64(ciphertextBuffer),
        nonce: bufferToBase64(nonce),
        key: key
    };
}

/**
 * Decrypts a base64 ciphertext using AES-256-GCM.
 * @param {string} ciphertextB64 - The base64-encoded ciphertext.
 * @param {string} nonceB64 - The base64-encoded nonce/IV.
 * @param {CryptoKey} key - The AES-GCM CryptoKey used for encryption.
 * @returns {Promise<string>} The decrypted plaintext string.
 */
export async function decrypt(ciphertextB64, nonceB64, key) {
    const ciphertext = base64ToBuffer(ciphertextB64);
    const nonce = base64ToBuffer(nonceB64);

    const decryptedBuffer = await crypto.subtle.decrypt(
        {
            name: "AES-GCM",
            iv: nonce
        },
        key,
        ciphertext
    );

    const decoder = new TextDecoder();
    return decoder.decode(decryptedBuffer);
}

/**
 * Exports a CryptoKey to a base64url-encoded string (raw format).
 * @param {CryptoKey} key - The AES-GCM CryptoKey to export.
 * @returns {Promise<string>} The base64url-encoded raw key bytes.
 */
export async function exportKey(key) {
    const rawKey = await crypto.subtle.exportKey("raw", key);
    return bufferToBase64Url(rawKey);
}

/**
 * Imports a CryptoKey from a base64url-encoded string (raw format).
 * @param {string} base64urlKey - The base64url-encoded raw key bytes.
 * @returns {Promise<CryptoKey>} The imported AES-GCM CryptoKey.
 */
export async function importKey(base64urlKey) {
    const rawKey = base64UrlToBuffer(base64urlKey);
    return await crypto.subtle.importKey(
        "raw",
        rawKey,
        "AES-GCM",
        true, // extractable
        ["encrypt", "decrypt"]
    );
}
