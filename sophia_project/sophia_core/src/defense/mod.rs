use std::process::{Command, exit};
use sc::syscall;
use libc::{PR_SET_PDEATHSIG, SIGKILL};
use std::ptr;

pub fn check_ptrace() -> bool {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("TracerPid:") {
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
    println!("[CRITICAL] Debugger detected or environment compromised.");

    let mut critical_region = vec![0u8; 1024];

    println!("*Initiating multi-pass memory wipe...*");
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

    let _ = Command::new("logger")
        .arg("-s")
        .arg("-p")
        .arg("user.error")
        .arg("Error: Corrupted configuration file in /etc/fonts/fonts.conf")
        .spawn();

    println!("*Injecting false error logs...*");
    exit(139);
}

pub fn set_self_destruct() {
    unsafe {
        syscall!(PRCTL, PR_SET_PDEATHSIG, SIGKILL, 0, 0, 0);
    }
}
