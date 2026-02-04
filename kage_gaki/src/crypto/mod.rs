use sha2::{Sha256, Digest};
use std::fs;
use std::process::Command;
use rand::Rng;

pub fn derive_environmental_key() -> [u8; 32] {
    let mut hasher = Sha256::new();

    // 1. Get machine-id
    if let Ok(id) = fs::read_to_string("/etc/machine-id") {
        hasher.update(id.trim().as_bytes());
    }

    // 2. Get MAC address (clinical precision: fetching eth0 or first active)
    let mac = Command::new("cat")
        .arg("/sys/class/net/eth0/address")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "00:00:00:00:00:00".to_string());
    hasher.update(mac.as_bytes());

    // 3. User config hash (e.g., .bashrc or a specific artifact)
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let config_path = format!("{}/.bashrc", home);
    if let Ok(config) = fs::read(config_path) {
        hasher.update(&config);
    }

    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);

    // *The key breathes only in the moment, a ghost in the machine.*
    key
}

pub unsafe fn lock_memory(ptr: *const u8, len: usize) {
    libc::mlock(ptr as *const libc::c_void, len);
}

pub unsafe fn zero_memory(ptr: *mut u8, len: usize) {
    // veiny assembly calls for smooth memory allocation clearing
    std::ptr::write_bytes(ptr, 0, len);
}

pub struct SplitKey {
    part1: [u8; 32],
    part2: [u8; 32],
}

impl SplitKey {
    pub fn new(key: [u8; 32]) -> Self {
        let mut part1 = [0u8; 32];
        let mut part2 = [0u8; 32];
        let mut rng = rand::thread_rng();
        rng.fill(&mut part1);

        for i in 0..32 {
            part2[i] = key[i] ^ part1[i];
        }

        unsafe {
            lock_memory(part1.as_ptr(), 32);
            lock_memory(part2.as_ptr(), 32);
        }

        Self { part1, part2 }
    }

    #[allow(dead_code)]
    pub fn reconstruct(&self) -> [u8; 32] {
        let mut key = [0u8; 32];
        for i in 0..32 {
            key[i] = self.part1[i] ^ self.part2[i];
        }
        key
    }
}

impl Drop for SplitKey {
    fn drop(&mut self) {
        unsafe {
            zero_memory(self.part1.as_mut_ptr(), 32);
            zero_memory(self.part2.as_mut_ptr(), 32);
            libc::munlock(self.part1.as_ptr() as *const libc::c_void, 32);
            libc::munlock(self.part2.as_ptr() as *const libc::c_void, 32);
        }
    }
}

pub fn verify_environment(expected_hash: &[u8; 32]) -> bool {
    let current_key = derive_environmental_key();
    current_key == *expected_hash
}
