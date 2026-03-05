with open("src/handlers/create.rs", "r") as f:
    content = f.read()

content = content.replace(
    "if ttl < MIN_TTL_SECONDS || ttl > MAX_TTL_SECONDS {",
    "if !(MIN_TTL_SECONDS..=MAX_TTL_SECONDS).contains(&ttl) {"
)

with open("src/handlers/create.rs", "w") as f:
    f.write(content)
