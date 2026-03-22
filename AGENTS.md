# AGENTS.md — Agent Operating Manual

This document serves exclusively as an operating manual for AI agents working on the Ignite project.

## Single Source of Truth for Architecture
For understanding system architecture, design decisions, and core invariants, **you must read the documents in `evolution/foundations/`**. That directory is the single source of truth for the system architecture.

## Non-Negotiable Security Invariants
These are absolute and non-negotiable. Every agent must respect these regardless of task:

1. **Atomic destructive reads:** The burn handler MUST use a single `DELETE FROM secrets WHERE id = ? RETURNING ...` query. Never a separate SELECT + DELETE. This is the core correctness guarantee.
2. **Server-side blindness:** The server must never log, store, or have access to plaintext secrets or decryption keys. Only ciphertext and nonces touch the backend.
3. **URL fragment isolation:** Decryption keys are carried in URL fragments (`#`). No code should ever send the fragment to the server.
4. **Error semantics:** 410 Gone = already burned or expired. 400 = malformed input. 404 is never used for secrets (to avoid leaking existence info via timing).
5. **No plaintext in logs:** Tracing/logging must never include secret payloads, ciphertext, nonces, or keys. Request IDs and metadata only.

## Frontend Conventions
- **Views**: New views go in `frontend/src/views/`.
- **Reusable Components**: Place them in `frontend/src/components/`.
- **API Client Logic**: Put all API interactions in `frontend/src/api.rs`.

*Note: Do not rely on this file for deep architectural or directory structure descriptions. See `evolution/`.*