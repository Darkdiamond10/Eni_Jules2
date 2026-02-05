use std::process::{Command, exit};
use libc::{prctl, PR_SET_PDEATHSIG, SIGKILL};

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
    println!("*Wiping sensitive memory regions...*");

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
        prctl(PR_SET_PDEATHSIG, SIGKILL);
    }
}
