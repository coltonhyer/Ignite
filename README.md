# 🔥 Ignite — Burn After Reading

[![CI](https://github.com/coltonhyer/ignite/actions/workflows/main.yml/badge.svg)](https://github.com/coltonhyer/ignite/actions/workflows/main.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A secure, local-first secret sharing service where secrets are **permanently destroyed on first read**.

Secrets are encrypted in your browser before they ever touch the server. The server stores only ciphertext and has **zero access to your plaintext** at any point. When a secret is read, it's atomically deleted from the database in the same operation — no race conditions, no second chances.

## Agentic Engineering Journey

This project served as my first attempt at making use of the new agentic engineering paradigm taking shape. I split my approach into two different workflows:

*   **Backend (Handoff to Jules Agents):** I wrote the backend by handing small tasks to Jules agents to build out individual units. The advantage here was that it was incredibly easy to reign them in and keep them focused because I provided very narrow scopes for them to work within. However, because I was never looking directly at the code being generated, I never felt fully comfortable in the codebase.
*   **Frontend (Pair Programming with Google Antigravity):** For the frontend, I switched to Google Antigravity and took a more hands-on pair programming approach. I worked alongside the agent to build out features, which allowed me to stay closer to the code and be heavily involved in testing and decision-making. The downside, however, was that being in the nitty-gritty led me to get locked into perfectionism. I found myself thinking that since I was using AI to help me, the bare minimum output needed to be absolute perfection.

Upon finishing the project, I don't think I have found the "perfect" workflow yet for taking full advantage of the technical speedup that agents offer while consistently producing at a high quality. Fortunately, I can take the lessons learned from this dual approach and apply them to my next project, trying out new techniques along the way. In the meantime, I plan to continue applying agentic engineering to the evolution and maintenance of this project.

## How It Works

```
Alice                          Server                         Bob
  │                              │                              │
  ├─ encrypt(secret) ───────────►│                              │
  │  AES-256-GCM in browser      │◄── store ciphertext          │
  │                              │                              │
  ├─ build URL ──────────────────┤                              │
  │  /s/{id}#Base64Key           │                              │
  │                              │                              │
  ├─ share URL via Slack/email ──┼──────────────────────────────►│
  │                              │                              │
  │                              │◄── GET /s/{id} (no key!) ────┤
  │                              │    DELETE...RETURNING ────────►│
  │                              │                              │
  │                              │    (secret deleted forever)   ├─ decrypt with #fragment
  │                              │                              │  AES-256-GCM in browser
  │                              │                              ├─ read plaintext
```

The decryption key lives in the URL fragment (`#`), which **browsers never send to the server**. The server is a blind courier.

## Stack

- **Backend:** Rust (Axum + Tokio)
- **Database:** SQLite (WAL mode) via `sqlx`
- **Frontend:** Vanilla HTML/JS + Tailwind CSS
- **Encryption:** Client-side AES-256-GCM (Web Crypto API)

## Quick Start

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- [Dioxus CLI](https://dioxuslabs.com/) (`cargo install dioxus-cli`)

### Run

**1. Start the Backend Server**
```bash
git clone https://github.com/coltonhyer/ignite.git
cd ignite
cargo run
```
Server starts at `http://localhost:3000` by default.

**2. Launch a Client**

* **Web Browser**: Run `dx build --release` inside the `frontend` directory, then navigate to `http://localhost:3000`.
* **Native Desktop App**: Open a new terminal, navigate into the `frontend` directory, and run the macOS native binary:
  ```bash
  cd frontend
  cargo run --bin frontend --features dioxus/desktop
  ```

### Configuration

| Env Var        | Default         | Description             |
|----------------|-----------------|-------------------------|
| `PORT`         | `3000`          | HTTP server port        |
| `DATABASE_URL` | `./ignite.db`   | SQLite database path    |

### Test

```bash
# Run unit and integration tests
cargo test
```

## API Reference

### `POST /api/secrets`

Create an encrypted secret.

**Request:**
```json
{
  "ciphertext": "<base64-encoded>",
  "nonce": "<base64-encoded>",
  "ttl_seconds": 3600
}
```

| Field          | Type   | Required | Default | Constraints          |
|----------------|--------|----------|---------|----------------------|
| `ciphertext`   | string | yes      | —       | ≤ 10KB after decode  |
| `nonce`        | string | yes      | —       | Base64-encoded       |
| `ttl_seconds`  | number | no       | 3600    | 300–86400            |

**Response (201):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "expires_at": "2026-03-01T22:00:00Z"
}
```

### `GET /api/secrets/:id`

Retrieve and **permanently destroy** a secret.

**Response (200):**
```json
{
  "ciphertext": "<base64-encoded>",
  "nonce": "<base64-encoded>"
}
```

| Status | Meaning                            |
|--------|------------------------------------|
| 200    | Secret retrieved (now destroyed)   |
| 400    | Invalid UUID format                |
| 410    | Already burned or expired          |
| 429    | Rate limited                       |

### `GET /health`

```json
{ "status": "ok", "db": "connected" }
```

## The Atomic Strategy

The core guarantee is that a secret can only ever be read once, even under high concurrency:

```sql
DELETE FROM secrets
WHERE id = ?1 AND expires_at > datetime('now')
RETURNING ciphertext, nonce
```

This single SQL statement atomically reads and deletes in one operation. SQLite's locking model ensures that if 100 requests arrive simultaneously for the same secret, **exactly one** gets the data — the rest get 410 Gone.

No separate SELECT + DELETE. No application-level locks. The database engine enforces the invariant.

## Rate Limits

| Endpoint            | Limit          |
|---------------------|----------------|
| `POST /api/secrets` | 10 req/min/IP  |
| `GET /api/secrets/` | 30 req/min/IP  |

Exceeding the limit returns `429 Too Many Requests` with a `Retry-After` header.

## Security Overview

Ignite is built on several non-negotiable security invariants. All contributions are expected to uphold these strictly:
1. **Atomic destructive reads:** Single `DELETE...RETURNING` operation for read.
2. **Server-side blindness:** The server never logs, stores, or touches plaintext.
3. **URL fragment isolation:** `#` fragments store decryption keys and never touch the backend.
4. **Opaque Error handling:** Use strict HTTP responses to avoid leaking existence information via timing side channels.

## Contributing

We welcome community contributions! Please read our [Agent Operating Manual (AGENTS.md)](AGENTS.md) and review the architectural documents in the `evolution/foundations/` directory before submitting pull requests to ensure alignment with our core invariants.

## License

[MIT](LICENSE)