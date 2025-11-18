pub fn encode(text: &str) -> String {
    unsafe { String::from_utf8_unchecked(encode_bytes(text.as_bytes())) }
}

pub fn encode_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    // for i in 0..bytes.len() {
    //     if i % 3 == 0 && i != 0 {
    //         result.push(b',');
    //     }
    result.push(bytes[0]);

    result
}
