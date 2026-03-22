fn main() {
    // Natively inject the absolute path of the backend crate's .sqlx directory
    // into the rustc compiler environment so that rustdoc doc-tests cannot
    // accidentally drop the relative workspace path during macro verification.
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        println!("cargo:rustc-env=SQLX_OFFLINE_DIR={}/.sqlx", manifest_dir);
    }
}
