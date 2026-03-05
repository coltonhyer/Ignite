#!/bin/bash
export DATABASE_URL=sqlite://ignite.db
# Initialize database
cargo run --bin ignite &
PID=$!
sleep 3
kill $PID
cargo check
