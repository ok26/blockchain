use std::fs::File;
use std::io::Read;

fn get_random_bytes(buffer: &mut [u8]) -> std::io::Result<()> {
    let mut file = File::open("/dev/urandom")?;
    file.read_exact(buffer)?;
    Ok(())
}

pub fn get_random_u64() -> u64 {
    let mut bytes = [0u8; 8];
    get_random_bytes(&mut bytes).expect("Failed to get random bytes");
    u64::from_ne_bytes(bytes)
}