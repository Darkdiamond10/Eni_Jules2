use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use std::fs;
use nix::sys::mman::{mlock, munlock};
use std::ptr;
use sophia_macros::encrypt_string;

pub struct GhostVault {
    key_part1: Vec<u8>,
    key_part2: Vec<u8>,
}

impl GhostVault {
    pub fn new() -> Self {
        let fingerprint = Self::derive_fingerprint();
        let salt_str = encrypt_string!("SOPHIA_SALT_V1");
        let salt = SaltString::encode_b64(salt_str.as_bytes()).unwrap();
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
            unsafe {
                let p1 = *key_part1.as_ptr().add(i);
                let mk = *master_key.as_ptr().add(i);
                *key_part2.as_mut_ptr().add(i) = p1 ^ mk;
            }
        }

        unsafe {
            mlock(key_part1.as_ptr() as *const libc::c_void, key_part1.len()).ok();
            mlock(key_part2.as_ptr() as *const libc::c_void, key_part2.len()).ok();

            for _i in 0..master_key.len() {
                ptr::write_volatile(master_key.as_ptr() as *mut u8, 0);
            }
        }

        Self { key_part1, key_part2 }
    }

    fn derive_fingerprint() -> String {
        let machine_id_path = encrypt_string!("/etc/machine-id");
        let machine_id = fs::read_to_string(machine_id_path)
            .unwrap_or_else(|_| encrypt_string!("unknown_machine"))
            .trim()
            .to_string();

        let user_env = encrypt_string!("USER");
        let user = std::env::var(user_env).unwrap_or_else(|_| encrypt_string!("unknown_user"));

        let mut mac = encrypt_string!("00:00:00:00:00:00");
        let ifaces = vec![
            encrypt_string!("eth0"),
            encrypt_string!("wlan0"),
            encrypt_string!("enp0s3"),
            encrypt_string!("ens33"),
        ];

        for iface in ifaces {
            let path = format!("{}/{}/{}", encrypt_string!("/sys/class/net"), iface, encrypt_string!("address"));
            if let Ok(m) = fs::read_to_string(path) {
                mac = m.trim().to_string();
                break;
            }
        }

        format!("{}-{}-{}", machine_id, user, mac)
    }

    pub fn decrypt(&self, ciphertext: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>, ()> {
        let mut master_key = [0u8; 32];

        for i in 0..32 {
            unsafe {
                let p1 = ptr::read_volatile(self.key_part1.as_ptr().add(i));
                let p2 = ptr::read_volatile(self.key_part2.as_ptr().add(i));
                ptr::write_volatile(master_key.as_mut_ptr().add(i), p1 ^ p2);
            }
        }

        let cipher = XChaCha20Poly1305::new(master_key.as_slice().into());
        let nonce = XNonce::from_slice(nonce_bytes);

        let result = cipher.decrypt(nonce, ciphertext).map_err(|_| ());

        unsafe {
            for i in 0..32 {
                ptr::write_volatile(master_key.as_mut_ptr().add(i), rand::random::<u8>());
                ptr::write_volatile(master_key.as_mut_ptr().add(i), 0);
            }
        }

        result
    }
}

impl Drop for GhostVault {
    fn drop(&mut self) {
        unsafe {
            for b in self.key_part1.iter_mut() { ptr::write_volatile(b, rand::random::<u8>()); ptr::write_volatile(b, 0); }
            for b in self.key_part2.iter_mut() { ptr::write_volatile(b, rand::random::<u8>()); ptr::write_volatile(b, 0); }
            munlock(self.key_part1.as_ptr() as *const libc::c_void, self.key_part1.len()).ok();
            munlock(self.key_part2.as_ptr() as *const libc::c_void, self.key_part2.len()).ok();
        }
    }
}
