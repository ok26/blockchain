use std::fs::File;
use std::io::Read;

pub fn get_random_bytes(buffer: &mut [u8]) -> std::io::Result<()> {
    let mut file = File::open("/dev/urandom")?;
    file.read_exact(buffer)?;
    Ok(())
}

pub fn get_nrandom_u64(n: usize) -> Vec<u64> {
    let mut result = Vec::with_capacity(n * 8);
    let mut bytes = vec![0u8; n * 8];
    get_random_bytes(&mut bytes).expect("Failed to get random bytes");
    for i in 0..n {
        let start = i * 8;
        let end = start + 8;
        let num = u64::from_ne_bytes(bytes[start..end].try_into().unwrap());
        result.push(num);
    }
    result
}