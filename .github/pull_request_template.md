## Description

<!-- Describe your changes in detail here. -->

## Security Invariants Checklist

Because Ignite relies on absolute security guarantees, please confirm your PR respects our core invariants (see [AGENTS.md](../AGENTS.md)):

- [ ] **Atomic Reads**: I have not altered the `DELETE...RETURNING` query logic or introduced a TOCTOU (Time-of-Check to Time-of-Use) bug.
- [ ] **Server Blindness**: I have not added any logging, tracing, or storage that could leak plaintext secrets or decryption keys.
- [ ] **URL Fragments**: I have not modified the frontend to send `#` fragments to the server.
- [ ] **Error Semantics**: Secret lookups return `400` for malformed input and `410` when missing, expired, or burned; they never return `404`.
- [ ] **Safe Logging**: I have not logged plaintext, ciphertext, nonces, keys, or secret payloads.

## Verification

- [ ] I have run `cargo test` and `cargo clippy`.
- [ ] I have verified the code changes work in standard workflows.
