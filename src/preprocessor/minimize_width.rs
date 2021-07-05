use std::collections::HashMap;

use crate::preprocessor::format_structs::PostFreq;

const MULTIPLIER: u32 = 2;
pub fn minimize_width(posts: &Vec<PostFreq>) -> Vec<PostFreq> {
    let mut word_freq_in_document: HashMap<String, Vec<u32>> = HashMap::new();
    posts.iter().for_each(|post| {
        post.term_frequency.iter().for_each(|(word, &freq)| {
            word_freq_in_document
                .entry(word.to_string())
                .or_insert(Vec::new())
                .push(freq)
        })
    });

    word_freq_in_document.iter_mut().for_each(|(_, freq)| {
        freq.sort();
        freq.dedup();
    });

    posts
        .into_iter()
        .map(|f| -> PostFreq {
            PostFreq {
                title: f.title.clone(),
                url: f.url.clone(),
                term_frequency: f
                    .term_frequency
                    .iter()
                    .map(|(string, &frequency)| -> (String, u32) {
                        (
                            string.to_string(),
                            (word_freq_in_document
                                .get(string)
                                .unwrap()
                                .binary_search(&frequency)
                                .ok()
                                .unwrap() as u32)
                                * MULTIPLIER
                                + 1,
                        )
                    })
                    .collect(),
            }
        })
        .collect()
}
