use std::collections::{HashMap, HashSet};
/// Does the following
/// - Remove non alphabetic characters
/// - Split on whitespace
/// - Convert to lowercase
/// - Remove stopwords, listed in stopwords.txt
/// - Create a counter, with words as key and their frequencies as value
pub fn tokenize(text: &String) -> HashMap<String, u32> {
    let stopwords = include_str!("../assets/stopwords.txt");
    let stopwords: HashSet<String> = stopwords.split_whitespace().map(String::from).collect();

    let mut counter: HashMap<String, u32> = HashMap::new();
    text.replace(|c: char| !c.is_alphabetic(), " ")
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|word| !stopwords.contains(word))
        .for_each(|word| *counter.entry(word).or_insert(0) += 1);
    counter
}