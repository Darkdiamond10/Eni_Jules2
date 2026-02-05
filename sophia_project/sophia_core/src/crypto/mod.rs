use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use std::fs;
use std::process::Command;
use nix::sys::mman::{mlock, munlock};
use std::ptr;

pub struct GhostVault {
    key_part1: Vec<u8>,
    key_part2: Vec<u8>,
}

impl GhostVault {
    pub fn new() -> Self {
        let fingerprint = Self::derive_fingerprint();
        // Fixed: Use encode_b64 to ensure the salt is valid for the password-hash crate
        let salt = SaltString::encode_b64(b"SOPHIA_SALT_V1").unwrap();
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(fingerprint.as_bytes(), &salt)
            .expect("Failed to derive key");

        let hash_bytes = password_hash.hash.expect("Failed to get hash bytes");
        let master_key = hash_bytes.as_bytes();

        let mut key_part1 = vec![0u8; 32];
        let mut key_part2 = vec![0u8; 32];

        for i in 0..32 {
            key_part1[i] = rand::random::<u8>();
            key_part2[i] = master_key[i] ^ key_part1[i];
        }

        unsafe {
            mlock(key_part1.as_ptr() as *const libc::c_void, key_part1.len()).ok();
            mlock(key_part2.as_ptr() as *const libc::c_void, key_part2.len()).ok();
        }

        Self { key_part1, key_part2 }
    }

    fn derive_fingerprint() -> String {
        let machine_id = fs::read_to_string("/etc/machine-id")
            .unwrap_or_else(|_| "unknown_machine".to_string())
            .trim()
            .to_string();

        let user = std::env::var("USER").unwrap_or_else(|_| "unknown_user".to_string());

        let mac = Command::new("cat")
            .arg("/sys/class/net/eth0/address")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "00:00:00:00:00:00".to_string());

        format!("{}-{}-{}", machine_id, user, mac)
    }

    pub fn decrypt(&self, ciphertext: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>, ()> {
        let mut master_key = [0u8; 32];
        for i in 0..32 {
            master_key[i] = self.key_part1[i] ^ self.key_part2[i];
        }

        let cipher = XChaCha20Poly1305::new(master_key.as_slice().into());
        let nonce = XNonce::from_slice(nonce_bytes);

        let result = cipher.decrypt(nonce, ciphertext).map_err(|_| ());

        unsafe {
            for i in 0..32 {
                ptr::write_volatile(master_key.as_mut_ptr().add(i), 0);
            }
        }

        result
    }
}

impl Drop for GhostVault {
    fn drop(&mut self) {
        unsafe {
            for b in self.key_part1.iter_mut() { ptr::write_volatile(b, 0); }
            for b in self.key_part2.iter_mut() { ptr::write_volatile(b, 0); }
            munlock(self.key_part1.as_ptr() as *const libc::c_void, self.key_part1.len()).ok();
            munlock(self.key_part2.as_ptr() as *const libc::c_void, self.key_part2.len()).ok();
        }
    }
}
