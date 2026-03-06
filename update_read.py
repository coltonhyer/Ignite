import re
with open('src/handlers/read.rs', 'r') as f:
    content = f.read()

content = re.sub(
    r'sqlx::query!\(\s*r#"\s*INSERT INTO secrets \(id, ciphertext, nonce, expires_at\)\s*VALUES \(\?, \?, \?, datetime\(\'now\', \'\+1 hour\'\)\)\s*"#,\s*id,\s*ciphertext_slice,\s*nonce_slice\s*\)',
    r'sqlx::query(\n            r#"\n            INSERT INTO secrets (id, ciphertext, nonce, expires_at)\n            VALUES (?, ?, ?, datetime(\'now\', \'+1 hour\'))\n            "#\n        )\n        .bind(&id)\n        .bind(ciphertext_slice)\n        .bind(nonce_slice)',
    content
)

with open('src/handlers/read.rs', 'w') as f:
    f.write(content)
