pub fn encode(bit_string: &String) -> String {
    let n_padded_bits = (15 - bit_string.len() % 15) % 15;
    let offset = 161;

    let mut bit_string = bit_string.clone();
    bit_string.push_str("0".repeat(n_padded_bits).as_str());

    let mut encoded: Vec<u16> = bit_string
        .as_bytes()
        .chunks_exact(15)
        .map(|fifteen_bits| {
            fifteen_bits
                .iter()
                .map(|x| *x as u16 - 48)
                .fold(0, |x, y| (x << 1) | y)
                + offset
        })
        .collect();

    let padding_char: u16 = format!("{:x}", n_padded_bits).chars().next().unwrap() as u16;
    encoded.insert(0, padding_char);
    std::char::decode_utf16(encoded.into_iter())
        .map(|result| result.unwrap())
        .collect()
}

pub fn decode(base2p15_encoded: &String) -> String {
    let offset = 0xa1;
    let padding_char: char = base2p15_encoded.chars().nth(0).unwrap();
    let n_padded_bits = padding_char.to_digit(16).unwrap();
    let mut decoded: String = base2p15_encoded
        .chars()
        .skip(1)
        .map(|c| format!("{:015b}", c as u32 - offset))
        .collect();
    (0..n_padded_bits).for_each(|_| {
        decoded.pop();
    });
    decoded
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    proptest! {

        #[test]
        fn proptest_reversible(bit_string in "[0-1]*") {
            println!("{}",bit_string);
            let s = decode(&encode(&bit_string));
            prop_assert_eq!(s,bit_string);
        }
    }
}