mod hash;

use hash::naive_hash;
use std::collections::HashMap;

/// Spectral Bloom Filter is a probabilistic data structure used to estimate frequency of an item in multiset
/// 
/// # Guarantees:
/// * The estimate never undershoots
/// * There are no false negatives
/// 
/// # Members
/// * n_hash_functions: The number of hash functions utilized by the filter
/// * sbf: The count vector
pub struct SpectralBloomFilter {
    n_hash_functions: u32,
    sbf: Vec<u32>,
}

impl SpectralBloomFilter {
    /// Create new Spectral Bloom Filter (SBF)
    ///
    /// # Arguments
    /// * tokens: Contains the tokens (possibly duplicate) to be inserted in the SBF
    /// * false_positive_rate: A configurable false positive rate in range \[0,1\]. Recommended value 0.1
    ///
    /// # Example
    ///
    /// ```
    /// use sbf::SpectralBloomFilter;
    /// let tokens: Vec<&str> = ["a", "b", "c", "d", "e", "e", "e"].to_vec();
    /// let sbf = SpectralBloomFilter::new(&tokens, 0.01);
    /// assert_eq!(sbf.get_frequency("e"), 3);
    /// ```
    pub fn new(tokens: &Vec<&str>, false_positive_rate: f32) -> Self {
        // Init counter
        let mut counter = HashMap::new();
        tokens
            .iter()
            .for_each(|token| *counter.entry(token).or_insert(0) += 1);

        // Compute optimal size
        let n_unique_tokens = counter.len() as u32;
        let (sbf_size, n_hash_functions) = Self::optimal_size(n_unique_tokens, false_positive_rate);

        // Create SBF of sbf_size
        let mut sbf: Vec<u32> = vec![0; sbf_size as usize];

        // Define function to insert item in SBF
        let insert_item = |&token| {
            let indices = Self::hash_indices(token, n_hash_functions as u64, sbf_size as u64);
            let minimum_value = indices.iter().map(|&i| sbf[i]).min().unwrap();
            indices.into_iter().for_each(|i| {
                if sbf[i] == minimum_value {
                    sbf[i] += 1;
                }
            });
        };

        // Fill SBF
        tokens.into_iter().for_each(insert_item);

        // Return
        SpectralBloomFilter {
            n_hash_functions: n_hash_functions,
            sbf: sbf,
        }
    }

    /// Given a token, return n indices that correspond to a location in sbf, where n = n_hash_functions
    ///
    /// # Arguments
    /// * token: The token that will be passed to the hash function
    /// * n_hash_functions: Then number of hash_functions
    /// * sbf_size: The size which will be used for modulo
    /// 
    fn hash_indices(token: &str, n_hash_functions: u64, sbf_size: u64) -> Vec<usize> {
        (0..n_hash_functions)
            .map(|i| (naive_hash(token, i) % sbf_size) as usize)
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
    /// If token does not exist, 0 will be returned
    ///
    /// # Guarantees
    /// * The estimate never undershoots
    /// * There are no false negatives
    ///
    /// # Arguments
    /// * token: The token whose frequency estimate is required
    ///
    /// # Example
    ///
    /// ```
    /// use sbf::SpectralBloomFilter;
    /// let tokens: Vec<&str> = ["a", "b", "c", "d", "e", "e", "e"].to_vec();
    /// let sbf = SpectralBloomFilter::new(&tokens, 0.01);
    /// assert_eq!(sbf.get_frequency("e"), 3);
    /// assert_eq!(sbf.get_frequency("xyz"),0);
    /// ```
    pub fn get_frequency(self: &Self, token: &str) -> u32 {
        let indices =
            Self::hash_indices(token, self.n_hash_functions as u64, self.sbf.len() as u64);
        indices.into_iter().map(|i| self.sbf[i]).min().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use super::*;
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