# Backend Foundation

## Purpose
Defines the core structure and abstractions for the Axum REST API and SQLite database interaction.

## Core Invariants
- Axum routing, SecretStore abstraction, and rate limiting handled inline in the router.
- Server is a "blind courier" — no access to plaintext.

## Mechanics / Data Flow
- Incoming requests hit the Axum router.
- Rate limits are checked inline.
- Requests interact with `SecretStore`, which wraps `SqlitePool`.

## Boundaries
- `SecretStore` is the strict boundary for all DB interactions, providing centralized query auditing and testability.
