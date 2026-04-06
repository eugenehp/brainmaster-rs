//! Example: read 4 seconds of EEG data from a Discovery.
use brainmaster::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let ports = DiscoveryDevice::find()?;
    if ports.is_empty() { eprintln!("No device found."); return Ok(()); }
    let mut device = DiscoveryDevice::open(&ports[0])?;
    println!("Connected to {}", ports[0]);
    let data = device.capture(SAMPLING_RATE as usize * 4)?;
    println!("Captured {} frames × {} channels", data.len(), NUM_CHANNELS);
    for (i, ch) in data.iter().take(5).enumerate() {
        println!("[{}] Fp1={:.2}µV O1={:.2}µV O2={:.2}µV", i, ch.data[0], ch.data[4], ch.data[13]);
    }
    device.close();
    Ok(())
}
