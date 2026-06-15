//! Cryptographic module
//!
//! Provides AES-256-GCM encryption, hierarchical key management,
//! Ed25519 signing for audit trails, and Argon2id password hashing.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub fn hex_encode(data: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut out = String::with_capacity(data.len() * 2);
    for b in data {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

pub fn hex_decode(s: &str) -> crate::Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return Err(crate::NewLokaError::Crypto("invalid hex length".into()));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for chunk in s.as_bytes().chunks_exact(2) {
        let hi = (chunk[0] as char)
            .to_digit(16)
            .ok_or_else(|| crate::NewLokaError::Crypto("invalid hex character".into()))?;
        let lo = (chunk[1] as char)
            .to_digit(16)
            .ok_or_else(|| crate::NewLokaError::Crypto("invalid hex character".into()))?;
        out.push((hi << 4 | lo) as u8);
    }
    Ok(out)
}
fn fill_random(buf: &mut [u8]) {
    rand::thread_rng().fill_bytes(buf);
}

/// Device Master Key (DMK) - top of the key hierarchy.
/// In production, derived from hardware-backed keystore where available.
#[derive(Debug, Clone)]
pub struct DeviceMasterKey {
    pub key: [u8; 32],
}

impl DeviceMasterKey {
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        fill_random(&mut key);
        Self { key }
    }

    pub fn derive_from_password(password: &str, salt: &[u8]) -> Self {
        let argon2 = Argon2::default();
        let salt_str = SaltString::encode_b64(salt).expect("valid salt");
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_str)
            .expect("argon2 hashing failed");
        let hash_bytes = password_hash.hash.expect("hash present");
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes.as_ref()[..32]);
        Self { key }
    }
}

/// Patient Record Key (PRK) - encrypts individual patient records.
#[derive(Debug, Clone)]
pub struct PatientRecordKey {
    pub key: [u8; 32],
}

impl PatientRecordKey {
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        fill_random(&mut key);
        Self { key }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> crate::Result<(Vec<u8>, [u8; 12])> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| crate::NewLokaError::Crypto(e.to_string()))?;
        let mut nonce_bytes = [0u8; 12];
        fill_random(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| crate::NewLokaError::Crypto(e.to_string()))?;
        Ok((ciphertext, nonce_bytes))
    }

    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> crate::Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| crate::NewLokaError::Crypto(e.to_string()))?;
        let nonce = Nonce::from_slice(nonce);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| crate::NewLokaError::Crypto(e.to_string()))?;
        Ok(plaintext)
    }
}

/// Hierarchical key encryption: DMK wraps PRK.
pub fn wrap_prk(
    dmk: &DeviceMasterKey,
    prk: &PatientRecordKey,
) -> crate::Result<(Vec<u8>, [u8; 12])> {
    let wrapper = PatientRecordKey { key: dmk.key };
    wrapper.encrypt(&prk.key)
}

pub fn unwrap_prk(
    dmk: &DeviceMasterKey,
    wrapped: &[u8],
    nonce: &[u8; 12],
) -> crate::Result<PatientRecordKey> {
    let wrapper = PatientRecordKey { key: dmk.key };
    let key_bytes = wrapper.decrypt(wrapped, nonce)?;
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(PatientRecordKey { key })
}

/// Audit signing key pair.
#[derive(Debug, Clone)]
pub struct AuditSigner {
    signing_key: SigningKey,
}

impl AuditSigner {
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        Self { signing_key }
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }

    pub fn verifying_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }
}

pub fn verify_audit_signature(
    verifying_key_bytes: &[u8],
    message: &[u8],
    signature_bytes: &[u8],
) -> crate::Result<bool> {
    let verifying_key = VerifyingKey::from_bytes(
        verifying_key_bytes
            .try_into()
            .map_err(|_| crate::NewLokaError::Crypto("invalid key length".into()))?,
    )
    .map_err(|e| crate::NewLokaError::Crypto(e.to_string()))?;
    let signature = Signature::from_bytes(
        signature_bytes
            .try_into()
            .map_err(|_| crate::NewLokaError::Crypto("invalid signature length".into()))?,
    );
    verifying_key
        .verify(message, &signature)
        .map(|_| true)
        .map_err(|e| crate::NewLokaError::Crypto(e.to_string()))
}

/// Hash a resource for integrity verification.
pub fn hash_resource(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_encode(&hasher.finalize())
}

pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    fill_random(&mut salt);
    salt
}
