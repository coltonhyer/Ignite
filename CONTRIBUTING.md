# Contributing to Ignite

Before changing code, read the [agent operating manual](AGENTS.md) and the architecture source of truth in [`evolution/foundations/`](evolution/foundations/).

## Non-Negotiable Security Invariants

All contributions must uphold these invariants:

1. **Atomic destructive reads:** Read and destroy with one `DELETE ... RETURNING` statement. Never split it into `SELECT` and `DELETE`.
2. **Server-side blindness:** The server never receives plaintext secrets or decryption keys.
3. **URL fragment isolation:** Decryption keys remain in the URL fragment and are never sent to the backend.
4. **Error semantics:** Return `400` for malformed input and `410` for missing, expired, or burned secrets. Never return `404` for secret lookups.
5. **Safe logging:** Never log plaintext, ciphertext, nonces, keys, or secret payloads.

## Pull Requests

Before opening a pull request, run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Then complete the pull request template, including its security checklist.
