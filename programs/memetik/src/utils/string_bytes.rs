pub fn string_to_fixed_bytes(s: &str, size: usize) -> [u8; 32] {
    let mut bytes = [0u8; 32]; // Initialize an array of the desired size filled with zeroes.
    let string_bytes = s.as_bytes(); // Convert the string to bytes.
    // Copy the string bytes into the fixed-size array, up to the maximum size.
    let len = string_bytes.len().min(size);
    bytes[..len].copy_from_slice(&string_bytes[..len]);

    bytes
}

pub fn fixed_bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}