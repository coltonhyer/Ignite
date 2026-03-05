import re

with open("src/router.rs", "r") as f:
    content = f.read()

# Add `post` to the routing imports
content = re.sub(
    r'use axum::\{routing::get, Router\};',
    'use axum::{routing::{get, post}, Router};',
    content
)

# Add the new route to the Router chain
route_addition = r"""
        // Temporary health check route to verify server is running
        .route("/health", get(crate::handlers::health::health_check))
        // API routes
        .route("/api/secrets", post(crate::handlers::create::create_secret))"""

content = re.sub(
    r'\s*// Temporary health check route to verify server is running\s*\.route\("/health", get\(crate::handlers::health::health_check\)\)',
    route_addition,
    content
)

with open("src/router.rs", "w") as f:
    f.write(content)
