use getrandom::getrandom;
use worker::{Date, Error};

pub fn generate_token() -> worker::Result<String> {
    let mut bytes = [0u8; 16];
    getrandom(&mut bytes).map_err(|e| Error::RustError(e.to_string()))?;
    Ok(hex::encode(bytes))
}

pub fn generate_id() -> worker::Result<String> {
    let mut bytes = [0u8; 8];
    getrandom(&mut bytes).map_err(|e| Error::RustError(e.to_string()))?;
    Ok(hex::encode(bytes))
}

pub fn now_ms() -> u64 {
    Date::now().as_millis()
}

pub fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}
