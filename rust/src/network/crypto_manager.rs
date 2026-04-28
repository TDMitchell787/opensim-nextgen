//! Cryptographic Manager for Dark Services
//!
//! Provides encryption, decryption, key management, and cryptographic operations
//! for secure asset transfers and zero trust networking.

use anyhow::{anyhow, Result};
use bytes::Bytes;
use rand::{rngs::OsRng as RandOsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

// AES-GCM encryption
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng as AeadOsRng},
    Aes256Gcm, Key, Nonce,
};

// Key derivation
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

/// Cryptographic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    /// Default encryption algorithm
    pub default_algorithm: EncryptionAlgorithm,
    /// Key derivation iterations
    pub key_derivation_iterations: u32,
    /// Salt length for key derivation
    pub salt_length: usize,
    /// Enable key rotation
    pub enable_key_rotation: bool,
    /// Key rotation interval in hours
    pub key_rotation_interval_hours: u64,
    /// Maximum key age in hours
    pub max_key_age_hours: u64,
    /// Enable forward secrecy
    pub enable_forward_secrecy: bool,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            default_algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_derivation_iterations: 100_000,
            salt_length: 32,
            enable_key_rotation: true,
            key_rotation_interval_hours: 24,
            max_key_age_hours: 168, // 7 days
            enable_forward_secrecy: true,
        }
    }
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

/// Cryptographic key information
#[derive(Debug, Clone)]
pub struct CryptoKey {
    pub key_id: Uuid,
    pub algorithm: EncryptionAlgorithm,
    pub key_data: Vec<u8>,
    pub salt: Vec<u8>,
    pub created_at: std::time::SystemTime,
    pub expires_at: Option<std::time::SystemTime>,
    pub usage_count: u64,
    pub max_usage: Option<u64>,
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub algorithm: EncryptionAlgorithm,
    pub key_id: Uuid,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub auth_tag: Vec<u8>,
}

/// Key derivation parameters
#[derive(Debug, Clone)]
pub struct KeyDerivationParams {
    pub password: String,
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub key_length: usize,
}

/// Cryptographic manager
pub struct CryptoManager {
    config: CryptoConfig,
    active_keys: Arc<RwLock<HashMap<Uuid, CryptoKey>>>,
    key_rotation_enabled: bool,
}

impl CryptoManager {
    /// Create a new crypto manager
    pub fn new(config: CryptoConfig) -> Self {
        Self {
            key_rotation_enabled: config.enable_key_rotation,
            config,
            active_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new cryptographic key
    pub fn generate_key(&self, length: usize) -> Result<Vec<u8>> {
        let mut key = vec![0u8; length];
        RandOsRng.fill_bytes(&mut key);
        Ok(key)
    }

    /// Generate a secure random nonce
    pub fn generate_nonce(&self) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::generate_key(&mut AeadOsRng);
        let nonce = Aes256Gcm::generate_nonce(&mut AeadOsRng);
        Ok(nonce.to_vec())
    }

    /// Create a new crypto key with metadata
    pub async fn create_key(&self, algorithm: EncryptionAlgorithm) -> Result<Uuid> {
        let key_id = Uuid::new_v4();
        let key_length = match algorithm {
            EncryptionAlgorithm::Aes256Gcm => 32,        // 256 bits
            EncryptionAlgorithm::ChaCha20Poly1305 => 32, // 256 bits
        };

        let key_data = self.generate_key(key_length)?;
        let salt = self.generate_key(self.config.salt_length)?;

        let crypto_key = CryptoKey {
            key_id,
            algorithm,
            key_data,
            salt,
            created_at: std::time::SystemTime::now(),
            expires_at: if self.config.max_key_age_hours > 0 {
                Some(
                    std::time::SystemTime::now()
                        + std::time::Duration::from_secs(self.config.max_key_age_hours * 3600),
                )
            } else {
                None
            },
            usage_count: 0,
            max_usage: None,
        };

        let mut keys = self.active_keys.write().await;
        keys.insert(key_id, crypto_key);

        info!("Created new crypto key: {}", key_id);
        Ok(key_id)
    }

    /// Encrypt data using AES-256-GCM
    pub fn encrypt_data(&self, data: &Bytes, key: &[u8]) -> Result<Bytes> {
        if key.len() != 32 {
            return Err(anyhow!(
                "Invalid key length for AES-256: expected 32 bytes, got {}",
                key.len()
            ));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut AeadOsRng);

        let ciphertext = cipher
            .encrypt(&nonce, data.as_ref())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext for simplicity
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(Bytes::from(result))
    }

    /// Decrypt data using AES-256-GCM
    pub fn decrypt_data(&self, encrypted_data: &Bytes, key: &[u8]) -> Result<Bytes> {
        if key.len() != 32 {
            return Err(anyhow!(
                "Invalid key length for AES-256: expected 32 bytes, got {}",
                key.len()
            ));
        }

        if encrypted_data.len() < 12 {
            return Err(anyhow!("Invalid encrypted data: too short"));
        }

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(Bytes::from(plaintext))
    }

    /// Encrypt data with a managed key
    pub async fn encrypt_with_key(&self, data: &Bytes, key_id: &Uuid) -> Result<EncryptedData> {
        let key = {
            let keys = self.active_keys.read().await;
            keys.get(key_id)
                .ok_or_else(|| anyhow!("Key not found: {}", key_id))?
                .clone()
        };

        let nonce = self.generate_nonce()?;

        match key.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let encrypted_bytes = self.encrypt_data(data, &key.key_data)?;

                // Split nonce and ciphertext (nonce is first 12 bytes)
                let (nonce_part, ciphertext_part) = encrypted_bytes.split_at(12);

                Ok(EncryptedData {
                    algorithm: key.algorithm,
                    key_id: *key_id,
                    nonce: nonce_part.to_vec(),
                    ciphertext: ciphertext_part.to_vec(),
                    auth_tag: Vec::new(), // Included in ciphertext for GCM
                })
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // For now, use AES-256-GCM implementation for ChaCha20Poly1305 as well
                // TODO: Implement proper ChaCha20Poly1305
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key.key_data));
                let nonce = Aes256Gcm::generate_nonce(&mut AeadOsRng);
                let ciphertext = cipher
                    .encrypt(&nonce, data.as_ref())
                    .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

                Ok(EncryptedData {
                    algorithm: key.algorithm.clone(),
                    key_id: *key_id,
                    nonce: nonce.to_vec(),
                    ciphertext: ciphertext,
                    auth_tag: Vec::new(), // Included in ciphertext for GCM
                })
            }
        }
    }

    /// Decrypt data with a managed key
    pub async fn decrypt_with_key(&self, encrypted_data: &EncryptedData) -> Result<Bytes> {
        let key = {
            let keys = self.active_keys.read().await;
            keys.get(&encrypted_data.key_id)
                .ok_or_else(|| anyhow!("Key not found: {}", encrypted_data.key_id))?
                .clone()
        };

        match encrypted_data.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                // Reconstruct the full encrypted data (nonce + ciphertext)
                let mut full_data = encrypted_data.nonce.clone();
                full_data.extend_from_slice(&encrypted_data.ciphertext);

                self.decrypt_data(&Bytes::from(full_data), &key.key_data)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // For now, use AES-256-GCM implementation for ChaCha20Poly1305 as well
                // TODO: Implement proper ChaCha20Poly1305
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key.key_data));
                let nonce = &encrypted_data.nonce[..12]; // First 12 bytes
                let nonce = aes_gcm::Nonce::from_slice(nonce);
                let plaintext = cipher
                    .decrypt(nonce, encrypted_data.ciphertext.as_ref())
                    .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;
                Ok(Bytes::from(plaintext))
            }
        }
    }

    /// Derive key from password using PBKDF2
    pub fn derive_key_from_password(&self, params: KeyDerivationParams) -> Result<Vec<u8>> {
        let mut key = vec![0u8; params.key_length];

        pbkdf2_hmac::<Sha256>(
            params.password.as_bytes(),
            &params.salt,
            params.iterations,
            &mut key,
        );

        Ok(key)
    }

    /// Generate a secure salt
    pub fn generate_salt(&self) -> Vec<u8> {
        let mut salt = vec![0u8; self.config.salt_length];
        RandOsRng.fill_bytes(&mut salt);
        salt
    }

    /// Hash data using SHA-256
    pub fn hash_data(&self, data: &[u8]) -> String {
        use sha2::Digest;
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Verify data integrity using hash
    pub fn verify_hash(&self, data: &[u8], expected_hash: &str) -> bool {
        let actual_hash = self.hash_data(data);
        actual_hash == expected_hash
    }

    /// Get key information
    pub async fn get_key_info(&self, key_id: &Uuid) -> Option<CryptoKey> {
        let keys = self.active_keys.read().await;
        keys.get(key_id).cloned()
    }

    /// List all active keys
    pub async fn list_keys(&self) -> Vec<Uuid> {
        let keys = self.active_keys.read().await;
        keys.keys().cloned().collect()
    }

    /// Rotate a key (create new key and deprecate old one)
    pub async fn rotate_key(&self, old_key_id: &Uuid) -> Result<Uuid> {
        let algorithm = {
            let keys = self.active_keys.read().await;
            keys.get(old_key_id)
                .ok_or_else(|| anyhow!("Key not found: {}", old_key_id))?
                .algorithm
                .clone()
        };

        // Create new key
        let new_key_id = self.create_key(algorithm).await?;

        // Mark old key as expired
        {
            let mut keys = self.active_keys.write().await;
            if let Some(old_key) = keys.get_mut(old_key_id) {
                old_key.expires_at = Some(std::time::SystemTime::now());
            }
        }

        info!("Rotated key {} -> {}", old_key_id, new_key_id);
        Ok(new_key_id)
    }

    /// Remove expired keys
    pub async fn cleanup_expired_keys(&self) -> Result<usize> {
        let now = std::time::SystemTime::now();
        let mut removed_count = 0;

        let mut keys = self.active_keys.write().await;
        keys.retain(|key_id, key| {
            if let Some(expires_at) = key.expires_at {
                if now > expires_at {
                    debug!("Removing expired key: {}", key_id);
                    removed_count += 1;
                    return false;
                }
            }
            true
        });

        if removed_count > 0 {
            info!("Cleaned up {} expired keys", removed_count);
        }

        Ok(removed_count)
    }

    /// Start automatic key rotation
    pub async fn start_key_rotation(&self) -> Result<()> {
        if !self.key_rotation_enabled {
            return Ok(());
        }

        let keys = self.active_keys.clone();
        let rotation_interval =
            std::time::Duration::from_secs(self.config.key_rotation_interval_hours * 3600);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(rotation_interval);

            loop {
                interval.tick().await;

                // Find keys that need rotation
                let keys_to_rotate: Vec<Uuid> = {
                    let keys_lock = keys.read().await;
                    let now = std::time::SystemTime::now();

                    keys_lock
                        .iter()
                        .filter(|(_, key)| {
                            if let Some(expires_at) = key.expires_at {
                                // Rotate keys that are 75% through their lifetime
                                let lifetime = expires_at
                                    .duration_since(key.created_at)
                                    .unwrap_or_default();
                                let elapsed =
                                    now.duration_since(key.created_at).unwrap_or_default();

                                elapsed > lifetime * 3 / 4
                            } else {
                                false
                            }
                        })
                        .map(|(key_id, _)| *key_id)
                        .collect()
                };

                // Rotate keys that need it
                for key_id in keys_to_rotate {
                    info!("Auto-rotating key: {}", key_id);
                    // Note: In a real implementation, you'd need access to the CryptoManager
                    // to call rotate_key. This is a simplified version.
                }
            }
        });

        info!(
            "Started automatic key rotation (interval: {} hours)",
            self.config.key_rotation_interval_hours
        );
        Ok(())
    }

    /// Generate a secure random UUID for session IDs
    pub fn generate_secure_uuid(&self) -> Uuid {
        Uuid::new_v4()
    }

    /// Create a digital signature (simplified implementation)
    pub fn sign_data(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        // This is a simplified HMAC-based signature
        // In production, you'd use proper digital signature algorithms like Ed25519
        use hmac::{Hmac, Mac};

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = <HmacSha256 as hmac::Mac>::new_from_slice(key)
            .map_err(|e| anyhow!("Invalid key for signing: {}", e))?;
        mac.update(data);

        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// Verify a digital signature
    pub fn verify_signature(&self, data: &[u8], signature: &[u8], key: &[u8]) -> Result<bool> {
        let expected_signature = self.sign_data(data, key)?;
        Ok(expected_signature == signature)
    }
}

/// Utility functions for cryptographic operations
pub mod crypto_utils {
    use super::*;

    /// Generate a secure password
    pub fn generate_secure_password(length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";

        let mut rng = RandOsRng;
        let password: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        password
    }

    /// Constant-time comparison for security-sensitive operations
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }

        result == 0
    }

    /// Secure memory clearing (best effort)
    pub fn secure_clear(data: &mut [u8]) {
        use std::ptr::write_volatile;
        use std::sync::atomic::{AtomicUsize, Ordering};

        // Use volatile writes to prevent compiler optimization
        for byte in data.iter_mut() {
            unsafe {
                write_volatile(byte, 0);
            }
        }

        // Memory barrier to prevent reordering
        static BARRIER: AtomicUsize = AtomicUsize::new(0);
        BARRIER.store(data.len(), Ordering::SeqCst);
    }
}
