import { test } from 'node:test';
import assert from 'node:assert';
import { encrypt, decrypt, exportKey, importKey } from '../static/js/crypto.js';

test('encrypt -> decrypt round-trip', async () => {
    const originalText = "Hello World! This is a secret message 🕵️‍♂️";

    // Encrypt
    const { ciphertext, nonce, key } = await encrypt(originalText);

    assert.ok(ciphertext.length > 0, "Ciphertext should not be empty");
    assert.ok(nonce.length > 0, "Nonce should not be empty");
    assert.ok(key, "Key should be returned");

    // Decrypt
    const decryptedText = await decrypt(ciphertext, nonce, key);

    assert.strictEqual(decryptedText, originalText, "Decrypted text should match original");
});

test('encrypt produces different ciphertexts/nonces', async () => {
    const originalText = "Hello World!";

    const res1 = await encrypt(originalText);
    const res2 = await encrypt(originalText);

    assert.notStrictEqual(res1.nonce, res2.nonce, "Nonces should be unique");
    assert.notStrictEqual(res1.ciphertext, res2.ciphertext, "Ciphertexts should be unique due to different keys/nonces");
});

test('key export and import round-trip', async () => {
    const originalText = "Testing key export/import";

    const { ciphertext, nonce, key } = await encrypt(originalText);

    // Export
    const exportedKeyB64Url = await exportKey(key);
    assert.ok(exportedKeyB64Url.length > 0, "Exported key should not be empty");
    assert.ok(!exportedKeyB64Url.includes('+'), "base64url should not contain +");
    assert.ok(!exportedKeyB64Url.includes('/'), "base64url should not contain /");
    assert.ok(!exportedKeyB64Url.includes('='), "base64url should not contain padding =");

    // Import
    const importedKey = await importKey(exportedKeyB64Url);

    // Decrypt with imported key
    const decryptedText = await decrypt(ciphertext, nonce, importedKey);
    assert.strictEqual(decryptedText, originalText, "Decrypted text with imported key should match original");
});

test('failed decryption throws a clear error', async () => {
    const { ciphertext, nonce, key } = await encrypt("Secret data");

    // Test 1: Wrong key
    const wrongKey = await importKey(await exportKey(key));
    // Wait, the above makes the SAME key, let's generate a new one
    const newKey = await crypto.subtle.generateKey(
        { name: "AES-GCM", length: 256 },
        true,
        ["encrypt", "decrypt"]
    );

    await assert.rejects(
        async () => { await decrypt(ciphertext, nonce, newKey) },
        { message: "Decryption failed. The key or ciphertext may be invalid." }
    );

    // Test 2: Modified ciphertext (invalid padding/MAC)
    const modifiedCiphertext = ciphertext.substring(0, ciphertext.length - 4) + "AAAA";
    await assert.rejects(
        async () => { await decrypt(modifiedCiphertext, nonce, key) },
        { message: "Decryption failed. The key or ciphertext may be invalid." }
    );

    // Test 3: Invalid base64
    await assert.rejects(
        async () => { await decrypt("not-base64!", nonce, key) },
        { message: "Decryption failed. The key or ciphertext may be invalid." }
    );
});
