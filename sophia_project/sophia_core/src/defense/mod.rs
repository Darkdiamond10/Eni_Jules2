use std::process::{Command, exit};
use sc::syscall;
use libc::{PR_SET_PDEATHSIG, SIGKILL};
use std::ptr;
use sophia_macros::encrypt_string;

pub fn check_ptrace() -> bool {
    let status_path = encrypt_string!("/proc/self/status");
    if let Ok(status) = std::fs::read_to_string(status_path) {
        let tracer_key = encrypt_string!("TracerPid:");
        for line in status.lines() {
            if line.starts_with(&tracer_key) {
                let pid = line.split_whitespace().last().unwrap_or("0");
                if pid != "0" {
                    return true;
                }
            }
        }
    }
    false
}

pub fn trigger_cyanide_pill() {
    println!("{}", encrypt_string!("[CRITICAL] Debugger detected or environment compromised."));

    let mut critical_region = vec![0u8; 1024];

    println!("{}", encrypt_string!("*Initiating multi-pass memory wipe...*"));
    unsafe {
        let ptr = critical_region.as_mut_ptr();
        let len = critical_region.len();

        for i in 0..len {
            ptr::write_volatile(ptr.add(i), rand::random::<u8>());
        }

        for i in 0..len {
            let val = ptr::read_volatile(ptr.add(i));
            ptr::write_volatile(ptr.add(i), val ^ 0xAA);
        }

        for i in 0..len {
            ptr::write_volatile(ptr.add(i), 0);
        }
    }

    let logger_cmd = encrypt_string!("logger");
    let fake_error = encrypt_string!("Error: Corrupted configuration file in /etc/fonts/fonts.conf");
    let _ = Command::new(logger_cmd)
        .arg("-s")
        .arg("-p")
        .arg(encrypt_string!("user.error"))
        .arg(fake_error)
        .spawn();

    println!("{}", encrypt_string!("*Injecting false error logs...*"));
    exit(139);
}

pub fn set_self_destruct() {
    unsafe {
        syscall!(PRCTL, PR_SET_PDEATHSIG, SIGKILL, 0, 0, 0);
    }
}
