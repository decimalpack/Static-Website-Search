mod hash;

use hash::naive_hash;
use std::collections::HashMap;

pub struct SpectralBloomFilter {
    n_hash_functions: u32,
    arr: Vec<u32>,
}

impl SpectralBloomFilter {
    pub fn new(tokens: &Vec<&str>, false_positive_rate: f32) -> Self {
        // Init counter
        let mut counter = HashMap::new();
        tokens
            .iter()
            .for_each(|token| *counter.entry(token).or_insert(0) += 1);

        // Compute optimal size
        let n_unique_tokens = counter.len() as u32;
        let (sbf_size, n_hash_functions) = Self::optimal_size(n_unique_tokens, false_positive_rate);

        // Fill SBF
        let mut sbf: Vec<u32> = vec![0; sbf_size as usize];
        tokens
            .into_iter()
            .map(|token| Self::hash_indices(token, n_hash_functions as u64, sbf_size as u64))
            .for_each(|indices| {
                let mn = indices.iter().map(|&i| sbf[i]).min().unwrap();
                indices.into_iter().for_each(|i| {
                    if sbf[i] == mn {
                        sbf[i] += 1;
                    }
                });
            });

        // Return
        SpectralBloomFilter {
            n_hash_functions: n_hash_functions,
            arr: sbf,
        }
    }

    fn hash_indices(token: &str, n_hash_functions: u64, sbf_size: u64) -> Vec<usize> {
        (0..n_hash_functions)
            .map(|i| (naive_hash(token, i) % sbf_size) as usize)
            .collect()
    }
    fn optimal_size(n_unique_tokens: u32, false_positive_rate: f32) -> (u32, u32) {
        let sbf_size = -((n_unique_tokens as f32) * false_positive_rate.ln() / 2_f32.ln().powi(2));
        let n_hash_functions = (sbf_size / n_unique_tokens as f32) * 2_f32.ln();
        (sbf_size.ceil() as u32, n_hash_functions.ceil() as u32)
    }
    pub fn get_frequency(self: &Self, token: &str) -> u32 {
        let indices =
            Self::hash_indices(token, self.n_hash_functions as u64, self.arr.len() as u64);

        indices.into_iter().map(|i| self.arr[i]).min().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    #[test]
    fn hand_written() {
        let tokens: Vec<&str> = ["a", "b", "c", "d", "e", "e", "e"].to_vec();
        let sbf = SpectralBloomFilter::new(&tokens, 0.01);
        let mut counter = HashMap::new();
        tokens
            .iter()
            .for_each(|token| *counter.entry(token).or_insert(0) += 1);
        counter.iter().for_each(|(token, &frequency)| {
            let frq = sbf.get_frequency(token);
            assert_eq!(frq, frequency);
        });
    }
    proptest! {
        #[test]
        fn proptest_false_negatives(s in any::<Vec<String>>()) {
            // Even for high false positive rate (99%), there should not be any false negatives
            let tokens:Vec<&str> = s.iter().map(|string| string.as_str()).collect();

            let sbf = SpectralBloomFilter::new(&tokens, 0.99);
            let false_negatives = tokens.iter().filter(|token| sbf.get_frequency(token)==0).count();
            prop_assert_eq!(false_negatives,0);
        }

        #[test]
        fn proptest_undershoot(s in any::<Vec<String>>()){
            let mut counter = HashMap::new();
            let tokens:Vec<&str> = s.iter().map(|string| string.as_str()).collect();
            tokens
                .iter()
                .for_each(|token| *counter.entry(token).or_insert(0) += 1);

            let sbf = SpectralBloomFilter::new(&tokens, 0.99);
            let undershoot = counter.into_iter().filter(|(token,frequency)| sbf.get_frequency(token)<*frequency).count();
            prop_assert_eq!(undershoot,0);
        }
    }
}

fn main() {}
