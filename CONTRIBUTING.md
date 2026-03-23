# Contributing to Ignite

We love your input! We want to make contributing to this project as easy and transparent as possible.

## The Agent Operating Manual

Ignite was built utilizing an agentic engineering workflow. Before you dive into the code, you **must** read our [Agent Operating Manual (AGENTS.md)](AGENTS.md). 

`AGENTS.md` is the single source of truth for understanding the system architecture, design decisions, and core invariants of this project.

## Non-Negotiable Security Invariants

All contributions must strictly uphold the following invariants (as detailed in `AGENTS.md`):
1. **Atomic destructive reads:** Single `DELETE...RETURNING` operation for read. No exceptions.
2. **Server-side blindness:** The server never logs, stores, or touches plaintext.
3. **URL fragment isolation:** `#` fragments store decryption keys and never touch the backend.
4. **Opaque Error handling:** Use strict HTTP responses to avoid leaking existence information via timing side channels.

## Pull Requests

1. **Fork the repository** and create your branch from `main`.
2. **Ensure tests pass** before submitting. Your PR will run through our automated CI pipeline (`cargo test`, `cargo clippy`, and `cargo fmt`). 
3. **Fill out the Pull Request Template**, paying special attention to the security invariant checklist.
