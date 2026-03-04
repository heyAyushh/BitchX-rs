use std::collections::HashMap;

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use blowfish::Blowfish;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use sha2::{Digest, Sha256};

use crate::error::{BitchXError, Result};

type BlowfishCbcEnc = cbc::Encryptor<Blowfish>;
type BlowfishCbcDec = cbc::Decryptor<Blowfish>;

#[derive(Debug, Clone)]
pub enum CipherType {
    Blowfish,
    AesGcm,
}

#[derive(Debug)]
pub struct EncryptionKey {
    pub target: String,
    pub key: Vec<u8>,
    pub cipher: CipherType,
}

#[derive(Debug, Default)]
pub struct KeyStore {
    keys: HashMap<String, EncryptionKey>,
}

fn derive_key_32(key: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.finalize().into()
}

fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    let pad_len = block_size - (data.len() % block_size);
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat_n(pad_len as u8, pad_len));
    padded
}

fn pkcs7_unpad(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Err(BitchXError::Encryption("Empty data".into()));
    }
    let pad_len = *data.last().unwrap() as usize;
    if pad_len == 0 || pad_len > 8 || pad_len > data.len() {
        return Err(BitchXError::Encryption("Invalid padding".into()));
    }
    if !data[data.len() - pad_len..]
        .iter()
        .all(|&b| b == pad_len as u8)
    {
        return Err(BitchXError::Encryption("Invalid padding".into()));
    }
    Ok(data[..data.len() - pad_len].to_vec())
}

pub fn blowfish_encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    use cbc::cipher::block_padding::NoPadding;
    let iv = [0u8; 8];
    let padded = pkcs7_pad(plaintext, 8);
    let enc = BlowfishCbcEnc::new_from_slices(key, &iv)
        .map_err(|e| BitchXError::Encryption(format!("Blowfish init error: {e}")))?;
    let mut buf = vec![0u8; padded.len() + 8];
    buf[..padded.len()].copy_from_slice(&padded);
    let encrypted = enc
        .encrypt_padded_mut::<NoPadding>(&mut buf, padded.len())
        .map_err(|_| BitchXError::Encryption("Blowfish encrypt error".into()))?;
    Ok(encrypted.to_vec())
}

pub fn blowfish_decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    use cbc::cipher::block_padding::NoPadding;
    let iv = [0u8; 8];
    let dec = BlowfishCbcDec::new_from_slices(key, &iv)
        .map_err(|e| BitchXError::Encryption(format!("Blowfish init error: {e}")))?;
    let mut buf = ciphertext.to_vec();
    let decrypted = dec
        .decrypt_padded_mut::<NoPadding>(&mut buf)
        .map_err(|_| BitchXError::Encryption("Blowfish decrypt error".into()))?;
    pkcs7_unpad(decrypted)
}

pub fn aes_gcm_encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let key_bytes = derive_key_32(key);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| BitchXError::Encryption(format!("AES-GCM init error: {e}")))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| BitchXError::Encryption(format!("AES-GCM encrypt error: {e}")))?;

    let mut result = nonce.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

pub fn aes_gcm_decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        return Err(BitchXError::Encryption(
            "Ciphertext too short for AES-GCM".into(),
        ));
    }
    let key_bytes = derive_key_32(key);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| BitchXError::Encryption(format!("AES-GCM init error: {e}")))?;
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    cipher
        .decrypt(nonce, &ciphertext[12..])
        .map_err(|e| BitchXError::Encryption(format!("AES-GCM decrypt error: {e}")))
}

pub fn encode_for_irc(data: &[u8]) -> String {
    BASE64.encode(data)
}

pub fn decode_from_irc(data: &str) -> Result<Vec<u8>> {
    BASE64
        .decode(data)
        .map_err(|e| BitchXError::Encryption(format!("Base64 decode error: {e}")))
}

impl KeyStore {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn add_key(&mut self, target: &str, key: &[u8], cipher: CipherType) {
        let lower = target.to_ascii_lowercase();
        self.keys.insert(
            lower,
            EncryptionKey {
                target: target.to_string(),
                key: key.to_vec(),
                cipher,
            },
        );
    }

    pub fn remove_key(&mut self, target: &str) -> Option<EncryptionKey> {
        self.keys.remove(&target.to_ascii_lowercase())
    }

    pub fn get_key(&self, target: &str) -> Option<&EncryptionKey> {
        self.keys.get(&target.to_ascii_lowercase())
    }

    pub fn encrypt_message(&self, target: &str, plaintext: &str) -> Option<String> {
        let ek = self.keys.get(&target.to_ascii_lowercase())?;
        let encrypted = match ek.cipher {
            CipherType::Blowfish => blowfish_encrypt(&ek.key, plaintext.as_bytes()).ok()?,
            CipherType::AesGcm => aes_gcm_encrypt(&ek.key, plaintext.as_bytes()).ok()?,
        };
        Some(encode_for_irc(&encrypted))
    }

    pub fn decrypt_message(&self, target: &str, ciphertext: &str) -> Option<String> {
        let ek = self.keys.get(&target.to_ascii_lowercase())?;
        let data = decode_from_irc(ciphertext).ok()?;
        let decrypted = match ek.cipher {
            CipherType::Blowfish => blowfish_decrypt(&ek.key, &data).ok()?,
            CipherType::AesGcm => aes_gcm_decrypt(&ek.key, &data).ok()?,
        };
        String::from_utf8(decrypted).ok()
    }

    pub fn is_encrypted(&self, target: &str) -> bool {
        self.keys.contains_key(&target.to_ascii_lowercase())
    }

    pub fn list_keys(&self) -> Vec<&str> {
        self.keys.values().map(|k| k.target.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blowfish_roundtrip() {
        let key = b"secret_key";
        let plaintext = b"Hello, IRC!";
        let encrypted = blowfish_encrypt(key, plaintext).unwrap();
        let decrypted = blowfish_decrypt(key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_blowfish_wrong_key() {
        let key = b"correct_key";
        let wrong_key = b"wrong_key!!";
        let plaintext = b"secret message";
        let encrypted = blowfish_encrypt(key, plaintext).unwrap();
        let result = blowfish_decrypt(wrong_key, &encrypted);
        assert!(result.is_err() || result.unwrap() != plaintext);
    }

    #[test]
    fn test_blowfish_empty() {
        let key = b"test_key";
        let encrypted = blowfish_encrypt(key, b"").unwrap();
        let decrypted = blowfish_decrypt(key, &encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_blowfish_exact_block_size() {
        let key = b"testkey1";
        let plaintext = b"12345678"; // exactly 8 bytes
        let encrypted = blowfish_encrypt(key, plaintext).unwrap();
        let decrypted = blowfish_decrypt(key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_roundtrip() {
        let key = b"my_secret_key";
        let plaintext = b"Hello, secure IRC!";
        let encrypted = aes_gcm_encrypt(key, plaintext).unwrap();
        let decrypted = aes_gcm_decrypt(key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_wrong_key() {
        let key = b"correct_key";
        let wrong_key = b"wrong_key!!";
        let plaintext = b"secret message";
        let encrypted = aes_gcm_encrypt(key, plaintext).unwrap();
        let result = aes_gcm_decrypt(wrong_key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_gcm_tampered_ciphertext() {
        let key = b"my_key";
        let plaintext = b"message";
        let mut encrypted = aes_gcm_encrypt(key, plaintext).unwrap();
        if let Some(last) = encrypted.last_mut() {
            *last ^= 0xFF;
        }
        assert!(aes_gcm_decrypt(key, &encrypted).is_err());
    }

    #[test]
    fn test_aes_gcm_short_ciphertext() {
        let key = b"key";
        assert!(aes_gcm_decrypt(key, &[0u8; 5]).is_err());
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello World!";
        let encoded = encode_for_irc(data);
        let decoded = decode_from_irc(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_invalid() {
        assert!(decode_from_irc("!!!not-valid-base64!!!").is_err());
    }

    #[test]
    fn test_keystore_add_and_get() {
        let mut ks = KeyStore::new();
        ks.add_key("#channel", b"secret", CipherType::Blowfish);
        assert!(ks.get_key("#channel").is_some());
        assert!(ks.get_key("#CHANNEL").is_some());
    }

    #[test]
    fn test_keystore_remove() {
        let mut ks = KeyStore::new();
        ks.add_key("#channel", b"secret", CipherType::Blowfish);
        let removed = ks.remove_key("#CHANNEL");
        assert!(removed.is_some());
        assert!(ks.get_key("#channel").is_none());
    }

    #[test]
    fn test_keystore_is_encrypted() {
        let mut ks = KeyStore::new();
        assert!(!ks.is_encrypted("#channel"));
        ks.add_key("#channel", b"key", CipherType::AesGcm);
        assert!(ks.is_encrypted("#channel"));
        assert!(ks.is_encrypted("#CHANNEL"));
    }

    #[test]
    fn test_keystore_list_keys() {
        let mut ks = KeyStore::new();
        ks.add_key("#a", b"k1", CipherType::Blowfish);
        ks.add_key("#b", b"k2", CipherType::AesGcm);
        let keys = ks.list_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"#a"));
        assert!(keys.contains(&"#b"));
    }

    #[test]
    fn test_keystore_encrypt_decrypt_blowfish() {
        let mut ks = KeyStore::new();
        ks.add_key("#secret", b"password123", CipherType::Blowfish);
        let encrypted = ks.encrypt_message("#secret", "Hello!").unwrap();
        let decrypted = ks.decrypt_message("#secret", &encrypted).unwrap();
        assert_eq!(decrypted, "Hello!");
    }

    #[test]
    fn test_keystore_encrypt_decrypt_aes() {
        let mut ks = KeyStore::new();
        ks.add_key("Alice", b"shared_secret", CipherType::AesGcm);
        let encrypted = ks.encrypt_message("Alice", "Top secret!").unwrap();
        let decrypted = ks.decrypt_message("Alice", &encrypted).unwrap();
        assert_eq!(decrypted, "Top secret!");
    }

    #[test]
    fn test_keystore_encrypt_unknown_target() {
        let ks = KeyStore::new();
        assert!(ks.encrypt_message("#unknown", "test").is_none());
    }

    #[test]
    fn test_keystore_decrypt_unknown_target() {
        let ks = KeyStore::new();
        assert!(ks.decrypt_message("#unknown", "dGVzdA==").is_none());
    }

    #[test]
    fn test_keystore_default() {
        let ks = KeyStore::default();
        assert!(!ks.is_encrypted("anything"));
    }

    #[test]
    fn test_pkcs7_pad_unpad() {
        let data = b"Hello";
        let padded = pkcs7_pad(data, 8);
        assert_eq!(padded.len(), 8);
        assert_eq!(padded[5], 3);
        assert_eq!(padded[6], 3);
        assert_eq!(padded[7], 3);
        let unpadded = pkcs7_unpad(&padded).unwrap();
        assert_eq!(unpadded, data);
    }

    #[test]
    fn test_pkcs7_pad_exact_block() {
        let data = b"12345678";
        let padded = pkcs7_pad(data, 8);
        assert_eq!(padded.len(), 16);
        let unpadded = pkcs7_unpad(&padded).unwrap();
        assert_eq!(unpadded, data);
    }

    #[test]
    fn test_pkcs7_unpad_invalid() {
        assert!(pkcs7_unpad(&[]).is_err());
        assert!(pkcs7_unpad(&[0]).is_err());
        assert!(pkcs7_unpad(&[9, 9, 9, 9, 9, 9, 9, 9]).is_err());
    }
}
