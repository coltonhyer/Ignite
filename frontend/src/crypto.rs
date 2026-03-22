#[cfg(target_arch = "wasm32")]
pub mod web_crypto {
    use anyhow::{anyhow, Result};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    pub type CryptoKeyObj = web_sys::CryptoKey;

    pub async fn encrypt_text(plaintext: &str) -> Result<(String, String, CryptoKeyObj)> {
        use js_sys::{Object, Reflect, Uint8Array};
        use wasm_bindgen_futures::JsFuture;
        use web_sys::window;

        let window = window().ok_or(anyhow!("No window"))?;
        let crypto = window
            .crypto()
            .map_err(|_| anyhow!("No crypto attribute"))?;
        let subtle = crypto.subtle();

        let algo = Object::new();
        Reflect::set(&algo, &"name".into(), &"AES-GCM".into()).unwrap();
        Reflect::set(&algo, &"length".into(), &256.into()).unwrap();

        let key_promise = subtle
            .generate_key_with_object(
                &algo,
                true,
                &js_sys::Array::of2(&"encrypt".into(), &"decrypt".into()),
            )
            .map_err(|e| anyhow!("generate_key failed: {:?}", e))?;
        let key_val = JsFuture::from(key_promise)
            .await
            .map_err(|e| anyhow!("key promise rejected: {:?}", e))?;
        let key: web_sys::CryptoKey = key_val.into();

        let mut iv = [0u8; 12];
        getrandom::getrandom(&mut iv).map_err(|e| anyhow!("getrandom failed: {}", e))?;
        let iv_array = Uint8Array::from(&iv[..]);

        let encrypt_algo = Object::new();
        Reflect::set(&encrypt_algo, &"name".into(), &"AES-GCM".into()).unwrap();
        Reflect::set(&encrypt_algo, &"iv".into(), &iv_array).unwrap();

        let encoder = web_sys::TextEncoder::new().map_err(|_| anyhow!("No encoder"))?;
        let encoded_pt = encoder.encode_with_input(plaintext);
        let encoded_pt_js = Uint8Array::from(&encoded_pt[..]);

        let encrypt_promise = subtle
            .encrypt_with_object_and_buffer_source(&encrypt_algo, &key, &encoded_pt_js)
            .map_err(|_| anyhow!("encrypt_with_object_and_buffer_source failed"))?;
        let ciphertext_val = JsFuture::from(encrypt_promise)
            .await
            .map_err(|_| anyhow!("encrypt failed"))?;
        let ciphertext_buf = Uint8Array::new(&ciphertext_val);

        let mut ct_bytes = vec![0; ciphertext_buf.length() as usize];
        ciphertext_buf.copy_to(&mut ct_bytes);

        let ciphertext_b64 = URL_SAFE_NO_PAD.encode(&ct_bytes);
        let nonce_b64 = URL_SAFE_NO_PAD.encode(&iv);

        Ok((ciphertext_b64, nonce_b64, key))
    }

    pub async fn export_key(key: &CryptoKeyObj) -> Result<String> {
        use js_sys::{ArrayBuffer, Uint8Array};
        use wasm_bindgen_futures::JsFuture;
        use web_sys::window;

        let subtle = window().unwrap().crypto().unwrap().subtle();
        let export_promise = subtle
            .export_key("raw", key)
            .map_err(|_| anyhow!("Export failed"))?;
        let exported_val = JsFuture::from(export_promise)
            .await
            .map_err(|_| anyhow!("Export promise rejected"))?;

        let buffer = ArrayBuffer::from(exported_val);
        let uint8_arr = Uint8Array::new(&buffer);
        let mut key_bytes = vec![0; uint8_arr.length() as usize];
        uint8_arr.copy_to(&mut key_bytes);

        Ok(URL_SAFE_NO_PAD.encode(&key_bytes))
    }

    pub async fn import_key(b64url_key: &str) -> Result<CryptoKeyObj> {
        use js_sys::{Object, Reflect, Uint8Array};
        use wasm_bindgen_futures::JsFuture;
        use web_sys::window;

        let key_bytes = URL_SAFE_NO_PAD.decode(b64url_key)?;
        let key_array = Uint8Array::from(&key_bytes[..]);

        let subtle = window().unwrap().crypto().unwrap().subtle();

        let import_algo = Object::new();
        Reflect::set(&import_algo, &"name".into(), &"AES-GCM".into()).unwrap();

        let import_promise = subtle
            .import_key_with_object(
                "raw",
                &key_array,
                &import_algo,
                true,
                &js_sys::Array::of2(&"encrypt".into(), &"decrypt".into()),
            )
            .map_err(|_| anyhow!("Import failed"))?;

        let imported_val = JsFuture::from(import_promise)
            .await
            .map_err(|_| anyhow!("Import promise rejected"))?;
        Ok(imported_val.into())
    }

    pub async fn decrypt_text(
        ciphertext_b64: &str,
        nonce_b64: &str,
        key: &CryptoKeyObj,
    ) -> Result<String> {
        use js_sys::{ArrayBuffer, Object, Reflect, Uint8Array};
        use wasm_bindgen_futures::JsFuture;
        use web_sys::window;

        let ct_bytes = URL_SAFE_NO_PAD.decode(ciphertext_b64)?;
        let nonce_bytes = URL_SAFE_NO_PAD.decode(nonce_b64)?;

        let subtle = window().unwrap().crypto().unwrap().subtle();

        let ct_arr = Uint8Array::from(&ct_bytes[..]);
        let nonce_arr = Uint8Array::from(&nonce_bytes[..]);

        let decrypt_algo = Object::new();
        Reflect::set(&decrypt_algo, &"name".into(), &"AES-GCM".into()).unwrap();
        Reflect::set(&decrypt_algo, &"iv".into(), &nonce_arr).unwrap();

        let decrypt_promise = subtle
            .decrypt_with_object_and_buffer_source(&decrypt_algo, key, &ct_arr)
            .map_err(|_| anyhow!("Decrypt failed"))?;

        let plain_val = JsFuture::from(decrypt_promise)
            .await
            .map_err(|_| anyhow!("Decrypt promise rejected"))?;
        let plain_buf = ArrayBuffer::from(plain_val);
        let plain_arr = Uint8Array::new(&plain_buf);

        let mut plain_bytes = vec![0; plain_arr.length() as usize];
        plain_arr.copy_to(&mut plain_bytes);

        String::from_utf8(plain_bytes).map_err(|e| anyhow::anyhow!("UTF8 Error: {}", e))
    }
}
#[cfg(target_arch = "wasm32")]
pub use web_crypto::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod native_crypto {
    use anyhow::{anyhow, Result};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    pub type CryptoKeyObj = Vec<u8>;

    pub async fn encrypt_text(plaintext: &str) -> Result<(String, String, CryptoKeyObj)> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use rand::Rng;

        let mut key_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut key_bytes);
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);

        let cipher = Aes256Gcm::new(key);

        let mut iv_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut iv_bytes);
        let nonce_obj = Nonce::from_slice(&iv_bytes);

        let ciphertext = cipher
            .encrypt(nonce_obj, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {:?}", e))?;

        let ciphertext_b64 = URL_SAFE_NO_PAD.encode(&ciphertext);
        let nonce_b64 = URL_SAFE_NO_PAD.encode(iv_bytes);

        Ok((ciphertext_b64, nonce_b64, key_bytes.to_vec()))
    }

    pub async fn export_key(key: &CryptoKeyObj) -> Result<String> {
        Ok(URL_SAFE_NO_PAD.encode(key))
    }

    pub async fn import_key(b64url_key: &str) -> Result<CryptoKeyObj> {
        let key_bytes = URL_SAFE_NO_PAD.decode(b64url_key)?;
        Ok(key_bytes)
    }

    pub async fn decrypt_text(ciphertext: &str, nonce: &str, key: &CryptoKeyObj) -> Result<String> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        let cipher =
            Aes256Gcm::new_from_slice(key).map_err(|e| anyhow!("Invalid key length: {:?}", e))?;

        let nonce_bytes = URL_SAFE_NO_PAD.decode(nonce)?;
        if nonce_bytes.len() != 12 {
            return Err(anyhow!("Invalid nonce span"));
        }
        let nonce_obj = Nonce::from_slice(&nonce_bytes);

        let ct_bytes = URL_SAFE_NO_PAD.decode(ciphertext)?;

        let plaintext_bytes = cipher
            .decrypt(nonce_obj, ct_bytes.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {:?}", e))?;

        String::from_utf8(plaintext_bytes).map_err(|e| anyhow!("Invalid UTF-8: {:?}", e))
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub use native_crypto::*;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_native_encryption_roundtrip() {
        let plaintext = "Clean and pragmatic test payload!";

        let (ciphertext_b64, nonce_b64, key_obj) =
            encrypt_text(plaintext).await.expect("Encryption failed");

        let exported_key_b64 = export_key(&key_obj).await.expect("Export failed");

        let imported_key_obj = import_key(&exported_key_b64).await.expect("Import failed");

        let decrypted_text = decrypt_text(&ciphertext_b64, &nonce_b64, &imported_key_obj)
            .await
            .expect("Decryption failed");

        assert_eq!(decrypted_text, plaintext);
    }

    #[tokio::test]
    async fn test_native_decryption_wrong_key() {
        let plaintext = "Clean and pragmatic test payload!";
        let (ciphertext_b64, nonce_b64, _key_obj) =
            encrypt_text(plaintext).await.expect("Encryption failed");

        let mut wrong_key = [0u8; 32];
        wrong_key[0] = 1;

        let result = decrypt_text(&ciphertext_b64, &nonce_b64, &wrong_key.to_vec()).await;
        assert!(result.is_err(), "Decryption should fail with a wrong key");
    }

    #[tokio::test]
    async fn test_native_decryption_tampered_ciphertext() {
        let plaintext = "Clean and pragmatic test payload!";
        let (mut ciphertext_b64, nonce_b64, key_obj) =
            encrypt_text(plaintext).await.expect("Encryption failed");

        if ciphertext_b64.starts_with('A') {
            ciphertext_b64.replace_range(0..1, "B");
        } else {
            ciphertext_b64.replace_range(0..1, "A");
        }

        let result = decrypt_text(&ciphertext_b64, &nonce_b64, &key_obj).await;
        assert!(
            result.is_err(),
            "Decryption should fail with tampered ciphertext"
        );
    }

    #[tokio::test]
    async fn test_native_decryption_invalid_base64() {
        let invalid_b64 = "This is not valid base64!@#$";
        let valid_nonce = "1234567890123456";
        let dummy_key = vec![0u8; 32];

        let result = decrypt_text(invalid_b64, valid_nonce, &dummy_key).await;
        assert!(
            result.is_err(),
            "Decryption should fail with invalid base64 ciphertext"
        );

        let valid_ciphertext = "validb64";
        let invalid_nonce_b64 = "not valid!@#";
        let result2 = decrypt_text(valid_ciphertext, invalid_nonce_b64, &dummy_key).await;
        assert!(
            result2.is_err(),
            "Decryption should fail with invalid base64 nonce"
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn test_web_encryption_roundtrip() {
        let plaintext = "Clean and pragmatic test payload!";
        let (ciphertext_b64, nonce_b64, key_obj) =
            encrypt_text(plaintext).await.expect("Encryption failed");

        let exported_key = export_key(&key_obj).await.expect("Export failed");
        let imported_key = import_key(&exported_key).await.expect("Import failed");

        let decrypted_text = decrypt_text(&ciphertext_b64, &nonce_b64, &imported_key)
            .await
            .expect("Decryption failed");

        assert_eq!(decrypted_text, plaintext);
    }
}
