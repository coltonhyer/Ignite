# 🔥 Ignite — Burn After Reading

A secure, local-first secret sharing service where secrets are **permanently destroyed on first read**.

Secrets are encrypted in your browser before they ever touch the server. The server stores only ciphertext and has **zero access to your plaintext** at any point. When a secret is read, it's atomically deleted from the database in the same operation — no race conditions, no second chances.

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

### Run

**1. Start the Backend Server (Required)**
```bash
git clone https://github.com/coltonhyer/ignite.git
cd ignite
cargo run
```
Server starts at `http://localhost:3000` by default.

**2. Launch the Client**
Because the architecture is heavily isolated, you must build or launch your preferred client interface:

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
# Unit + integration tests
cargo test

# Concurrency stress test (100 simultaneous requests × 10 iterations)
cargo test stress -- --nocapture
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

## Concurrency Test Results

_Run `cargo test stress -- --nocapture` and paste results here after Phase 5._

## License

[MIT](LICENSE)