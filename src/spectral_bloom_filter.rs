mod murmur3;

use murmur3::murmurhash3_x86_32 as hash_fn;
use std::collections::HashMap;
use std::fmt;

/// Spectral Bloom Filter is a probabilistic data structure used to estimate frequency of an item in multiset
///
/// **Warning:** In case of integer overflow, the value is set to MAX of coresponding integer type
///
/// # Guarantees:
/// * The estimate never undershoots
/// * There are no false negatives
///
/// # Members
/// * n_hash_functions: The number of hash functions utilized by the filter
/// * sbf: The count vector
///
/// # Example
///
/// ```
/// use spectral_bloom_filter::SpectralBloomFilter;
/// use std::collections::HashMap;
///
/// let mut hash_map: HashMap<String, u32> = HashMap::new();
/// hash_map.insert("a".to_string(), 5);
///
/// // Create a SBF with false_positive_rate = 1% and width = 4 (max frequency = 2^4 - 1 = 15)
/// let sbf = SpectralBloomFilter::new(&hash_map, 0.01, 4);
///
/// assert_eq!(sbf.get_frequency(&"a".to_string()), 5);
/// assert_eq!(sbf.get_frequency(&"x".to_string()), 0);
/// ```
#[derive(fmt::Debug)]
pub struct SpectralBloomFilter {
    pub n_hash_functions: u32,
    pub sbf: Vec<u32>,
    pub width: u32,
}

impl SpectralBloomFilter {
    /// Create new Spectral Bloom Filter (SBF)
    ///
    /// # Arguments
    /// * counter: Multiset represented as HashMap with elements as key, frequencies as value
    /// * false_positive_rate: A configurable false positive rate in range \[0,1\]. Recommended value 0.1
    /// * width: Number of bits to represent frequency in SBF. Overshooting counter frequencies will be automatically converted to 2^width-1
    pub fn new(counter: &HashMap<String, u32>, false_positive_rate: f32, width: u32) -> Self {
        // Compute optimal size
        let (sbf_size, n_hash_functions) =
            Self::optimal_size(counter.keys().count() as u32, false_positive_rate);

        // Create SBF of sbf_size
        let mut sbf: Vec<u32> = vec![0; sbf_size as usize];

        // Define function to insert item in SBF
        let insert_item = |(key, &frequency)| {
            let indices = Self::hash_indices(key, n_hash_functions, sbf_size);
            let minimum_value = std::cmp::min(
                indices.iter().map(|&i| sbf[i]).min().unwrap(),
                2u32.pow(width) - 1,
            );
            // In case of overflow, set to MAX value
            let minimum_value = match minimum_value.checked_add(frequency) {
                Some(v) => v,
                None => 2u32.pow(width) - 1,
            };
            indices.iter().for_each(|&i| {
                if sbf[i] <= minimum_value {
                    sbf[i] = minimum_value;
                }
            });
        };

        // Fill SBF
        counter.into_iter().for_each(insert_item);

        // Return
        SpectralBloomFilter {
            n_hash_functions: n_hash_functions,
            sbf: sbf,
            width: width,
        }
    }

    /// Given a token, return n indices that correspond to a location in sbf, where n = n_hash_functions
    ///
    /// # Arguments
    /// * key: An element of the multiset / counter
    /// * n_hash_functions: Then number of hash_functions
    /// * sbf_size: The size which will be used for modulo
    fn hash_indices(key: &String, n_hash_functions: u32, sbf_size: u32) -> Vec<usize> {
        (0..n_hash_functions)
            .map(|i| (hash_fn(key.as_bytes(), i) % sbf_size) as usize)
            .collect()
    }

    /// Compute the optimal size using the formulae from
    ///
    /// https://stackoverflow.com/questions/658439/how-many-hash-functions-does-my-bloom-filter-need
    fn optimal_size(n_unique_tokens: u32, false_positive_rate: f32) -> (u32, u32) {
        let sbf_size = -((n_unique_tokens as f32) * false_positive_rate.ln() / 2_f32.ln().powi(2));
        let n_hash_functions = (sbf_size / n_unique_tokens as f32) * 2_f32.ln();
        (sbf_size.ceil() as u32, n_hash_functions.ceil() as u32)
    }

    /// Get the frequency estimate for a token
    ///
    /// # Returns
    /// * The frequency estimate, a non-zero positive number
    /// * 0 if token does not exist
    ///
    /// # Guarantees
    /// * The estimate never undershoots
    /// * There are no false negatives
    ///
    /// # Arguments
    /// * key: An element of the multiset / counter
    pub fn get_frequency(self: &Self, key: &String) -> u32 {
        let indices = Self::hash_indices(key, self.n_hash_functions, self.sbf.len() as u32);
        indices.into_iter().map(|i| self.sbf[i]).min().unwrap()
    }
    pub fn base2p15_encode(&self) -> String {
        let mut bit_string = self
            .sbf
            .iter()
            .map(|&x| format!("{:0width$b}", x, width = self.width as usize))
            .fold(String::new(), |x, y| format!("{}{}", x, y));
        let n_padded_bits = (15 - bit_string.len() % 15) % 15;
        let offset = 161;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    #[test]
    fn hand_written() {
        let mut hash_map: HashMap<String, u32> = HashMap::new();
        hash_map.insert("a".to_string(), 1);
        hash_map.insert("b".to_string(), 2);
        hash_map.insert("c".to_string(), 32);
        let sbf = SpectralBloomFilter::new(&hash_map, 0.01, 4);
        hash_map.iter().for_each(|(token, &frequency)| {
            let frq = sbf.get_frequency(token);
            assert_eq!(frq, frequency);
        });
    }
    proptest! {
        #[test]
        fn proptest_false_negatives(counter in any::<HashMap<String,u32>>()) {
            // Even for high false positive rate (99%), there should not be any false negatives
            let sbf = SpectralBloomFilter::new(&counter, 0.99,4);
            let false_negatives = counter.keys().filter(|token| sbf.get_frequency(token)==0).count();
            prop_assert_eq!(false_negatives,0);
        }

        #[test]
        fn proptest_undershoot(counter in any::<HashMap<String,u32>>()){
            let sbf = SpectralBloomFilter::new(&counter, 0.99,4);
            let undershoot = counter.into_iter().filter(|(token,frequency)| sbf.get_frequency(token)<*frequency).count();
            prop_assert_eq!(undershoot,0);
        }
    }
}
