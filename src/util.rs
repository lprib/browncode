//big endian
pub fn append_u32(vec: &mut Vec<u8>, n: u32) {
    vec.push((n >> 24) as u8);
    vec.push((n >> 16) as u8);
    vec.push((n >> 8) as u8);
    vec.push(n as u8);
}
