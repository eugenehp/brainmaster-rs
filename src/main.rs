//! CLI: scan for Discovery devices, connect, and stream EEG data.

use brainmaster::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("BrainMaster Discovery — Rust CLI");
    println!("================================\n");

    println!("Scanning for serial ports...");
    let ports = DiscoveryDevice::find()?;
    if ports.is_empty() {
        eprintln!("No serial ports found. Is the Discovery connected via USB?");
        return Ok(());
    }

    println!("Found {} port(s):", ports.len());
    for (i, p) in ports.iter().enumerate() {
        println!("  [{}] {}", i, p);
    }

    println!("\nOpening {}...", ports[0]);
    let mut device = DiscoveryDevice::open(&ports[0])?;

    println!("Starting stream (256 Hz, 24 channels)...");
    device.start()?;

    let n = SAMPLING_RATE as usize * 4;
    println!("Capturing {} frames (~4 seconds)...\n", n);

    let mut count = 0;
    while count < n {
        match device.read_frame()? {
            Some(_frame) => {
                count += 1;
                if count % SAMPLING_RATE as usize == 0 || count <= 5 {
                    let ch = device.channels();
                    println!(
                        "[{:>6}] Fp1={:>8.2}µV  F3={:>8.2}µV  C3={:>8.2}µV  O1={:>8.2}µV  O2={:>8.2}µV",
                        count, ch.data[0], ch.data[1], ch.data[2], ch.data[4], ch.data[13]
                    );
                }
            }
            None => continue,
        }
    }

    device.stop()?;
    device.close();
    println!("\nDone — {} frames captured.", count);
    Ok(())
}
