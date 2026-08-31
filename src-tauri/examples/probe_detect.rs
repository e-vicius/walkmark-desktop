//! Calibration harness for step detection.
//!
//! Watches the primary display for a while and prints, for every sample, how
//! much the screen moved and whether a step would have been captured. Use it
//! when tuning `detect::threshold_for`: perform a normal workflow while it
//! runs, then check that the committed samples line up with the actions.
//!
//! `cargo run --release --example probe_detect -- [seconds] [sensitivity]`

use std::time::Duration;

fn main() {
    let mut args = std::env::args().skip(1);
    let seconds: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(20);
    let sensitivity: f32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(0.55);

    let sources = match walkmark_lib::probe::list_sources(false) {
        Ok(s) => s,
        Err(e) => return println!("listing failed: {e}"),
    };
    let Some(monitor) = sources.iter().find(|s| s.id.starts_with("monitor:")) else {
        return println!("no monitors found");
    };

    println!(
        "watching {} for {seconds}s at sensitivity {sensitivity} (threshold {:.4})",
        monitor.name,
        walkmark_lib::probe::threshold_for(sensitivity)
    );

    let samples = match walkmark_lib::probe::trace(
        &monitor.id,
        Duration::from_secs(seconds),
        Duration::from_millis(600),
        sensitivity,
        Duration::from_millis(1200),
    ) {
        Ok(s) => s,
        Err(e) => return println!("trace failed: {e}"),
    };

    for s in &samples {
        println!(
            "{:>6}ms  activity {:.4}  drift {:.4}  {}",
            s.at.as_millis(),
            s.activity,
            s.drift,
            if s.committed { "STEP" } else { "" }
        );
    }
    println!(
        "\n{} samples, {} steps",
        samples.len(),
        samples.iter().filter(|s| s.committed).count()
    );
}
