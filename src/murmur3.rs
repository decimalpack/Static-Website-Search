/*
The MIT License (MIT)

Copyright (c) 2015 Magnus Hallin

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

// https://github.com/mhallin/murmurhash3-rs/blob/202fbc5b74859a723d9cbce05d32370eb730411e/src/mmh3_32.rs
use std::mem;

fn fmix32(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;

    return h;
}

fn get_32_block(bytes: &[u8], index: usize) -> u32 {
    let b32: &[u32] = unsafe { mem::transmute(bytes) };

    return b32[index];
}

pub fn murmurhash3_x86_32(bytes: &[u8], seed: u32) -> u32 {
    let c1 = 0xcc9e2d51u32;
    let c2 = 0x1b873593u32;
    let read_size = 4;
    let len = bytes.len() as u32;
    let block_count = len / read_size;

    let mut h1 = seed;

    for i in 0..block_count as usize {
        let mut k1 = get_32_block(bytes, i);

        k1 = k1.wrapping_mul(c1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(c2);

        h1 ^= k1;
        h1 = h1.rotate_left(13);
        h1 = h1.wrapping_mul(5);
        h1 = h1.wrapping_add(0xe6546b64)
    }
    let mut k1 = 0u32;

    if len & 3 == 3 {
        k1 ^= (bytes[(block_count * read_size) as usize + 2] as u32) << 16;
    }
    if len & 3 >= 2 {
        k1 ^= (bytes[(block_count * read_size) as usize + 1] as u32) << 8;
    }
    if len & 3 >= 1 {
        k1 ^= bytes[(block_count * read_size) as usize + 0] as u32;
        k1 = k1.wrapping_mul(c1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(c2);
        h1 ^= k1;
    }

    h1 ^= bytes.len() as u32;
    h1 = fmix32(h1);

    return h1;
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
