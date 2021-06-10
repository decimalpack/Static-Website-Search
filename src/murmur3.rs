pub fn murmurhash3_x86_32(bytes: &[u8], seed: u32) -> u32 {
    // https://en.wikipedia.org/wiki/MurmurHash
    let c1 = 0xcc9e2d51;
    let c2 = 0x1b873593;
    let r1 = 15;
    let r2 = 13;
    let m = 5;
    let n = 0xe6546b64;

    if bytes.len() == 0 {
        return 0;
    }

    let mut hash: u32 = seed;

    for four_byte_chunk in bytes.chunks(4) {
        let mut arr = [0u8; 4];
        four_byte_chunk
            .iter()
            .enumerate()
            .for_each(|(i, &x)| arr[i] = x);
        let mut k: u32 = u32::from_le_bytes(arr);
        k = k.wrapping_mul(c1);
        k = k.rotate_left(r1);
        k = k.wrapping_mul(c2);

        hash ^= k;
        if four_byte_chunk.len() == 4 {
            hash = hash.rotate_left(r2);
            hash = hash.wrapping_mul(m).wrapping_add(n);
        }
    }

    hash ^= bytes.len() as u32;
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85ebca6b);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0xc2b2ae35);
    hash ^= hash >> 16;
    hash
}
#[cfg(test)]
mod test {
    use super::murmurhash3_x86_32;

    #[test]
    fn test_empty_string() {
        assert!(murmurhash3_x86_32("".as_bytes(), 0) == 0);
    }

    #[test]
    fn test_tail_lengths() {
        assert!(murmurhash3_x86_32("1".as_bytes(), 0) == 2484513939);
        assert!(murmurhash3_x86_32("12".as_bytes(), 0) == 4191350549);
        assert!(murmurhash3_x86_32("123".as_bytes(), 0) == 2662625771);
        assert!(murmurhash3_x86_32("1234".as_bytes(), 0) == 1914461635);
    }

    #[test]
    fn test_large_data() {
        assert!(murmurhash3_x86_32("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam at consequat massa. Cras eleifend pellentesque ex, at dignissim libero maximus ut. Sed eget nulla felis".as_bytes(), 0)
            == 1004899618);
    }
}
