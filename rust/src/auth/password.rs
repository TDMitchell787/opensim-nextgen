use anyhow::{Result, anyhow};
use md5::Digest;
use tracing::{debug, error};

pub struct PasswordAuth;

impl PasswordAuth {
    pub fn verify_password(plain_password: &str, hashed_password: &str) -> Result<bool> {
        debug!("Verifying password for login");
        
        let hash_part = if hashed_password.starts_with("$1$") {
            &hashed_password[3..]
        } else {
            hashed_password
        };
        
        let digest = md5::compute(plain_password.as_bytes());
        let plain_hash = format!("{:x}", digest);
        
        let is_valid = plain_hash == hash_part;
        
        if is_valid {
            debug!("Password verification successful");
        } else {
            debug!("Password verification failed");
        }
        
        Ok(is_valid)
    }
    
    pub fn hash_password(plain_password: &str) -> String {
        let digest = md5::compute(plain_password.as_bytes());
        format!("$1${:x}", digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_verification() {
        let plain_password = "testpassword";
        let hashed = PasswordAuth::hash_password(plain_password);
        
        assert!(PasswordAuth::verify_password(plain_password, &hashed).unwrap());
        assert!(!PasswordAuth::verify_password("wrongpassword", &hashed).unwrap());
    }
    
    #[test]
    fn test_md5_prefix_handling() {
        let plain_password = "test123";
        let expected_hash = "482c811da5d5b4bc6d497ffa98491e38";
        let prefixed_hash = format!("$1${}", expected_hash);
        
        assert!(PasswordAuth::verify_password(plain_password, &prefixed_hash).unwrap());
        assert!(PasswordAuth::verify_password(plain_password, expected_hash).unwrap());
    }
}