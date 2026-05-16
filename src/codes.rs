use getrandom::getrandom;
use worker::Error;

pub const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

pub fn generate_code() -> worker::Result<String> {
    let mut bytes = [0u8; 5];
    getrandom(&mut bytes).map_err(|e| Error::RustError(e.to_string()))?;
    Ok(bytes
        .iter()
        .map(|&b| ALPHABET[(b as usize) % ALPHABET.len()] as char)
        .collect())
}
