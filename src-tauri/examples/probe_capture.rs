//! Manual smoke test for the capture backend.
//!
//! Run with `cargo run --example probe_capture`. Verifies that sources can be
//! enumerated and that a real frame can be grabbed and hashed on this machine.

use std::time::Instant;

fn main() {
    println!("permission granted: {}", steppy_lib::probe::has_permission());

    let started = Instant::now();
    match steppy_lib::probe::list_sources(false) {
        Ok(sources) => {
            println!("{} sources in {:?}", sources.len(), started.elapsed());
            for source in sources.iter().take(12) {
                println!(
                    "  [{:?}] {} — {} ({}x{}) id={}",
                    source.kind, source.name, source.detail, source.width, source.height, source.id
                );
            }

            if let Some(monitor) = sources.iter().find(|s| s.id.starts_with("monitor:")) {
                let started = Instant::now();
                match steppy_lib::probe::grab(&monitor.id) {
                    Ok((w, h, elapsed_hash)) => println!(
                        "captured {w}x{h} in {:?} (signature in {elapsed_hash:?})",
                        started.elapsed()
                    ),
                    Err(e) => println!("capture failed: {e}"),
                }
            }
        }
        Err(e) => println!("listing failed: {e}"),
    }
}
