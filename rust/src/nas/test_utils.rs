#[cfg(test)]
pub fn unhexlify(byte_str: &str) -> Vec<u8> {
    (0..byte_str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&byte_str[i..i + 2], 16).unwrap())
        .collect()
}
