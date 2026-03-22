# Security Foundation

## Purpose
Defines the absolute non-negotiable security invariant structure for Ignite.

## Core Invariants
- AES-256-GCM client-side encryption.
- "Blind courier" server blindness (no plaintext on the server).
- Atomic `DELETE...RETURNING` destruction on the read route.
- URL fragment (`#`) key isolation.

## Mechanics / Data Flow
- Keys are never sent to the backend.
- Destruction is guaranteed to happen only once, with no separate SELECT and DELETE queries.

## Boundaries
- The server boundary only sees ciphertext and nonces. The rest happens in the client.
