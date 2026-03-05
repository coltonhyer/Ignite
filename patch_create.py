with open("src/handlers/create.rs", "r") as f:
    content = f.read()

content = content.replace(
    "use base64::engine::general_purpose::STANDARD;",
    "use base64::{engine::general_purpose::STANDARD, Engine};"
)

with open("src/handlers/create.rs", "w") as f:
    f.write(content)
