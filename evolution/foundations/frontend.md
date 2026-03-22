# Frontend Foundation

## Purpose
Defines the structure and delivery mechanism for the Dioxus SPA client.

## Core Invariants
- Dioxus SPA targeting WASM (web) and Desktop.
- Strict air-gap between frontend logic and backend.
- Decryption keys are held exlusively in the fragment (`#`), handled completely within the frontend.

## Mechanics / Data Flow
- The frontend handles UI state, client-side routing, and all cryptographic operations.
- API client logic interacts with the server without ever sending the decryption key.

## Boundaries
- Modular structure: `api/`, `components/`, `crypto/`, `views/`.
- Views belong in `frontend/src/views/`, reusable components in `frontend/src/components/`, API client logic in `frontend/src/api.rs`.
