# API Foundation

## Purpose
Defines constraints, schemas, and endpoints for external interactions.

## Core Invariants
- DELETE-only read route (`DELETE /api/secrets/{id}`) for querying and atomic destruction simultaneously.
- No `GET` route for secrets to protect against accidental burns from API crawlers.
- Strict payload and TTL constraints.

## Mechanics / Data Flow
- Max payload: 10KB (ciphertext, after base64 decoding).
- TTL range: 300s (5min) to 86400s (24h), default 3600s (1h).

## Boundaries
- Error envelope structure applies to all responses.
