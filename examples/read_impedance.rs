//! Example: read impedance values from a Discovery.
use brainmaster::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let ports = DiscoveryDevice::find()?;
    if ports.is_empty() { eprintln!("No device found."); return Ok(()); }
    let mut device = DiscoveryDevice::open(&ports[0])?;
    device.start()?;
    for _ in 0..SAMPLING_RATE as usize * 5 {
        if let Some(_) = device.read_frame()? {
            let imp = device.impedances();
            if imp.active.iter().any(|&v| v != 0.0) {
                println!("Active:  {:?}", &imp.active[..NUM_EEG_CHANNELS]);
                println!("Ref:     {:?}", &imp.reference[..NUM_EEG_CHANNELS]);
                println!();
            }
        }
    }
    device.stop()?;
    device.close();
    Ok(())
}
