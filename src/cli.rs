use serde::{Deserialize, Serialize};
use spectral_bloom_filter::SpectralBloomFilter;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize, Debug)]
struct Document {
    document_link: String,
    term_frequency: HashMap<String, u32>,
}

#[derive(Debug, Serialize)]
struct SearchItem {
    document_link: String,
    sbf_base2p15: String,
    width: u32,
    n_hash_functions: u32,
}

fn main() -> std::io::Result<()> {
    let file = File::open("tokens.json")?;
    let false_positive_rate = 0.01;
    let width = 4;
    let buf_reader = BufReader::new(file);
    let tokens_json: Vec<Document> = serde_json::from_reader(buf_reader)?;

    let search_index: Vec<SearchItem> = tokens_json
        .into_iter()
        .map(|document| {
            let sbf =
                SpectralBloomFilter::new(&document.term_frequency, false_positive_rate, width);
            SearchItem {
                sbf_base2p15: sbf.base2p15_encode(),
                width: sbf.width,
                n_hash_functions: sbf.n_hash_functions,
                document_link: document.document_link,
            }
        })
        .collect();
    let j = serde_json::to_string(&search_index)?;
    let template = std::fs::read_to_string("search_template.html")?;
    let search_page = template.replace("UNIQUE_SEARCH_INDEX_PLACEHOLDER", j.as_str());
    std::fs::write("search.html", search_page)
}
