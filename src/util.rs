/// push a u32 value to vec, in little endian format
pub fn append_u32(vec: &mut Vec<u8>, n: u32) {
    vec.extend_from_slice(&n.to_le_bytes())
}