/// AES-256-GCM 加密/解密工具，用于保护 AI API Key
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;

/// 从环境变量获取或生成加密密钥（32 bytes）
fn get_key() -> [u8; 32] {
    static mut KEY: Option<[u8; 32]> = None;
    unsafe {
        if let Some(k) = KEY {
            return k;
        }
    }
    let key: [u8; 32] = if let Ok(env_key) = std::env::var("MARKSHAREX_ENCRYPT_KEY") {
        let bytes = env_key.as_bytes();
        let mut k = [0u8; 32];
        let len = bytes.len().min(32);
        k[..len].copy_from_slice(&bytes[..len]);
        k
    } else {
        // No env var → generate random key (survives restarts via DB persistence)
        let mut k = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut k);
        tracing::warn!(
            "MARKSHAREX_ENCRYPT_KEY 未设置，已生成随机密钥。请设置环境变量以确保持久化。"
        );
        k
    };
    unsafe {
        KEY = Some(key);
    }
    key
}

pub fn encrypt(plaintext: &str) -> String {
    if plaintext.is_empty() {
        return String::new();
    }
    let key = get_key();
    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key");
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("encrypt");
    // nonce || ciphertext, base64 encoded
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    BASE64.encode(&combined)
}

#[allow(dead_code)]
pub fn decrypt(encoded: &str) -> String {
    if encoded.is_empty() {
        return String::new();
    }
    let combined = BASE64.decode(encoded).unwrap_or_default();
    if combined.len() < 12 {
        return String::new();
    }
    let key = get_key();
    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key");
    let nonce = Nonce::from_slice(&combined[..12]);
    cipher
        .decrypt(nonce, &combined[12..])
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original = "sk-test-key-12345";
        let encrypted = encrypt(original);
        assert!(!encrypted.is_empty());
        assert_ne!(encrypted, original);
        let decrypted = decrypt(&encrypted);
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_empty() {
        assert_eq!(encrypt(""), "");
        assert_eq!(decrypt(""), "");
    }
}
