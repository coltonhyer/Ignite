# CLAUDE.md — Code Review Directives

For project context, architecture, conventions, and API contracts, see **[AGENTS.md](./AGENTS.md)**.

This file contains directives specific to Claude's role as a **code reviewer** on this project.

## Review Priorities

When reviewing PRs, enforce these in order of severity:

### 🔴 Critical — Block the PR

- **Broken atomicity:** The burn handler (`GET /api/secrets/:id`) must use a single `DELETE...RETURNING` query. If you see a separate SELECT + DELETE, or an application-level lock wrapping two queries, block it. This is a correctness bug.
- **Plaintext leakage:** Any code that logs, stores, or transmits plaintext secrets, decryption keys, or raw payloads to the server. Check tracing calls, error messages, and debug output.
- **Key sent to server:** Any JS code that includes `window.location.hash` in a fetch request, or any server-side code that reads/expects a fragment or key parameter.
- **Missing error handling:** Handlers that return raw status codes or unwrap Results instead of using `AppError`.

### 🟡 Warning — Request changes

- **TOCTOU bugs:** Any read-then-act pattern on the secrets table that isn't a single atomic statement.
- **Timing side channels:** Code paths where the response time differs meaningfully between "never existed" and "already burned" — both should return 410 with similar latency.
- **Unbounded inputs:** Missing validation on payload size (10KB limit) or TTL range (300–86400s).
- **Logging secrets metadata:** Even ciphertext lengths or nonces in logs could leak information. Flag it.

### 🟢 Info — Suggest improvements

- Clippy warnings or idiomatic Rust improvements.
- Missing or inadequate tests for new functionality.
- Frontend accessibility or UX issues.
- Documentation gaps.