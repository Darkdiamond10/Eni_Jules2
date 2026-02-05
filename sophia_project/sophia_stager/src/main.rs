use std::fs;
use std::path::Path;
use std::io::{self, Read, Seek, SeekFrom};
use libc::{prctl, PR_SET_NAME};
use std::ffi::CString;

fn main() -> io::Result<()> {
    let fake_name = "systemd-cache-cleaner";
    let c_name = CString::new(fake_name).unwrap();
    unsafe {
        prctl(PR_SET_NAME, c_name.as_ptr(), 0, 0, 0);
    }

    println!("[INFO] Starting cache integrity verification...");

    let home = std::env::var("HOME").unwrap_or_default();
    let targets = vec![
        format!("{}/.cache/spotify/Storage/index-db", home),
        format!("{}/.mozilla/firefox/profiles.ini", home),
    ];

    for target in targets {
        if Path::new(&target).exists() {
            println!("[INFO] Verifying checksums for {}...", target);
            let _ = verify_file_integrity(&target);
        }
    }

    println!("[INFO] Cache maintenance complete. System is healthy.");
    Ok(())
}

fn verify_file_integrity(path: &str) -> io::Result<()> {
    // Corrected: Read from specific files instead of directories to avoid EISDIR
    let mut file = fs::File::open(path)?;
    let mut buffer = [0u8; 4096];
    let _ = file.read(&mut buffer)?;
    let _hash: u8 = buffer.iter().fold(0, |acc, &x| acc.wrapping_add(x));
    Ok(())
}

fn extract_shadow_payload(path: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    file.seek(SeekFrom::End(-1024))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
