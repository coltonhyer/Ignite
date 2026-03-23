## Description

<!-- Describe your changes in detail here. -->

## Security Invariants Checklist

Because Ignite relies on absolute security guarantees, please confirm your PR respects our core invariants (see [AGENTS.md](../AGENTS.md)):

- [ ] **Atomic Reads**: I have not altered the `DELETE...RETURNING` query logic or introduced a TOCTOU (Time-of-Check to Time-of-Use) bug.
- [ ] **Server Blindness**: I have not added any logging, tracing, or storage that could leak plaintext secrets or decryption keys.
- [ ] **URL Fragments**: I have not modified the frontend to send `#` fragments to the server.
- [ ] **Timing Security**: I have not introduced logic that meaningfully alters the response latency between a secret that never existed and one that was already burned.

## Verification

- [ ] I have ran `cargo test` and `cargo clippy`.
- [ ] I have verified the code changes work in standard workflows.
