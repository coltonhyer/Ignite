# 001-ignite

## Context/Problem

Retroactively capturing the architectural baseline and core pivots made for Ignite, a "Burn After Reading" secret sharing service.

## Decision

- **Pure Rust Stack:** Decoupled Dioxus SPA vs. Dioxus Fullstack. We rejected Fullstack due to strict security/air-gap boundary needs.
- **Burn API:** Decided on a DELETE-only endpoint (`DELETE /api/secrets/{id}`) for querying and atomic destruction simultaneously. We intentionally rejected a GET route to protect against API crawlers accidentally burning secrets.
- **Storage Abstraction:** Introduced `SecretStore` to wrap `SqlitePool` for testability and centralized query auditing.

## Alternatives Considered

- **Dioxus Fullstack:** Rejected to maintain a strict separation between front-end and back-end for security.
- **GET route for Burn API:** Rejected because link preview bots and crawlers would accidentally burn secrets upon fetching the page.

## Consequences

- The decoupling enforces server blindness and strict boundaries.
- The DELETE-only atomic action ensures correctness and protects secrets from bots.
- `SecretStore` provides a clear boundary for DB interactions.
