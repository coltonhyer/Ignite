# AGENTS.md — Project Context & Agent Directives

## Project Overview

**Ignite** is a "Burn After Reading" secret sharing service. Secrets are encrypted client-side (AES-256-GCM via Web Crypto API), stored as ciphertext in SQLite, and atomically destroyed on first read via `DELETE...RETURNING`.

The server is a **blind courier** — it never has access to plaintext. Decryption keys live exclusively in URL fragments (`#`), which browsers never send to the server.

**RFC:** https://www.notion.so/315c10e6389880769620eb9c9e2fce52

## Stack

- **Backend:** Rust (Axum + Tokio)
- **Database:** SQLite via `sqlx` (WAL mode)
- **Frontend:** Vanilla HTML5/JS + Tailwind CSS (CDN)
- **Encryption:** Client-side AES-256-GCM (Web Crypto API)

## Architecture Invariants

These are non-negotiable. Every agent must respect these regardless of task.

1. **Atomic destructive reads:** The burn handler MUST use a single `DELETE FROM secrets WHERE id = ? RETURNING ...` query. Never a separate SELECT + DELETE. This is the core correctness guarantee.
2. **Server-side blindness:** The server must never log, store, or have access to plaintext secrets or decryption keys. Only ciphertext and nonces touch the backend.
3. **URL fragment isolation:** Decryption keys are carried in URL fragments (`#`). No code should ever send the fragment to the server.
4. **Error semantics:** 410 Gone = already burned or expired. 400 = malformed input. 404 is never used for secrets (to avoid leaking existence info via timing).
5. **No plaintext in logs:** Tracing/logging must never include secret payloads, ciphertext, nonces, or keys. Request IDs and metadata only.

## Directory Structure

```
src/
├── main.rs           # Entry point, config, server startup
├── router.rs         # Route definitions + middleware stack
├── db.rs             # SQLite pool initialization
├── migrate.rs        # Boot-time migration runner
├── error.rs          # AppError enum + IntoResponse impl
├── handlers/
│   ├── health.rs     # GET /health
│   ├── create.rs     # POST /api/secrets
│   └── read.rs       # GET /api/secrets/:id (atomic burn)
├── middleware/
│   └── rate_limit.rs # Per-IP rate limiting
└── workers/
    └── expiry.rs     # TTL cleanup background task
static/
├── index.html        # SPA entry point
└── js/
    ├── app.js        # Client-side routing + reveal logic
    ├── crypto.js     # AES-256-GCM encrypt/decrypt
    └── url.js        # URL fragment key encoding
migrations/
└── 001_create_secrets.sql
tests/
├── atomicity.rs      # Correctness integration tests
└── stress.rs         # Concurrent load test
```

## Code Conventions

### Rust / Backend

- All handler errors use the `AppError` enum — never raw status codes or string responses.
- All handlers return `Result<impl IntoResponse, AppError>`.
- SQL queries use `sqlx::query!` or `sqlx::query_as!` macros where possible for compile-time verification.
- Background workers must handle graceful shutdown via `tokio::select!` on a `CancellationToken`.
- Rate limiting is per-IP via tower middleware.
- New routes are registered in `src/router.rs` via `Router::new().route(...)`.
- New handlers go in `src/handlers/` as separate files, one per endpoint.
- Use `tracing::info!`, `tracing::error!`, etc. for logging — never `println!`.

### Frontend / JavaScript

- No build step. Vanilla JS with ES module imports.
- Tailwind CSS via CDN (`<script src="https://cdn.tailwindcss.com">`).
- All crypto operations use the Web Crypto API (`crypto.subtle`) — no external crypto libraries.
- Base64url encoding (URL-safe: `+` → `-`, `/` → `_`, no padding) for URL fragment keys.

## Payload Constraints

- Max payload: 10KB (ciphertext, after base64 decoding)
- TTL range: 300s (5min) to 86400s (24h), default 3600s (1h)
- TTL presets for UI: 5 minutes, 15 minutes, 1 hour, 24 hours

## Database Schema

```sql
CREATE TABLE IF NOT EXISTS secrets (
    id TEXT PRIMARY KEY NOT NULL,
    ciphertext BLOB NOT NULL,
    nonce BLOB NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_secrets_expires_at ON secrets(expires_at);
```

## API Contract

### `POST /api/secrets`
- Request: `{ "ciphertext": "<base64>", "nonce": "<base64>", "ttl_seconds": <optional, 300-86400, default 3600> }`
- Success: 201 `{ "id": "<uuid>", "expires_at": "<iso8601>" }`
- Errors: 400 (invalid input), 413 (payload > 10KB), 429 (rate limited)

### `GET /api/secrets/:id`
- Success: 200 `{ "ciphertext": "<base64>", "nonce": "<base64>" }` (secret is now deleted)
- Errors: 400 (malformed UUID), 410 (already burned or expired), 429 (rate limited)

### `GET /health`
- Success: 200 `{ "status": "ok", "db": "connected" }`
- Failure: 503 `{ "status": "error", "db": "disconnected" }`

## Error Envelope

All error responses follow this format:

```json
{ "error": "<human-readable message>", "code": "<MACHINE_CODE>" }
```

Codes: `PAYLOAD_TOO_LARGE`, `INVALID_REQUEST`, `ALREADY_BURNED`, `NOT_FOUND`, `RATE_LIMITED`, `INTERNAL`

## How to Run

```bash
cargo run                          # Start server on :3000
cargo test                         # Unit + integration tests
cargo test stress -- --nocapture   # Concurrency stress test
cargo clippy -- -D warnings        # Lint
cargo fmt --check                  # Format check
```

## Environment Variables

| Var            | Default       | Description           |
|----------------|---------------|-----------------------|
| `PORT`         | `3000`        | HTTP server port      |
| `DATABASE_URL` | `./ignite.db` | SQLite database path  |