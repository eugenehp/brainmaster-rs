//! Example: scan for Discovery serial ports.
use brainmaster::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let ports = DiscoveryDevice::find()?;
    println!("Found {} port(s):", ports.len());
    for p in &ports { println!("  {}", p); }
    Ok(())
}
