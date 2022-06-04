use clap::Parser;
use serde::{Deserialize, Serialize};
use static_website_search::compressor::base2p15;
use static_website_search::estimator::spectral_bloom_filter::SpectralBloomFilter;
use static_website_search::preprocessor::naive;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize, Debug)]
struct Post {
    title: String,
    url: String,
    body: String,
}

#[derive(Debug, Serialize)]
struct SearchItem {
    url: String,
    title: String,
    sbf_base2p15: String,
    width: u32,
    size: u32,
    n_hash_functions: u32,
}

#[derive(Parser, Debug)]
#[clap(version = "0.1", about = "https://github.com/decimalpack/Static-Website-Search")]
struct Opts {
    /// The tokens.json file from which to read.
    #[clap(short, long)]
    tokens_file: String,

    /// The false positive rate, lower rate means higher sizes
    ///
    /// High values will introduce more false positives in search results
    #[clap(short, long, default_value = "0.01")]
    false_positive_rate: f32,

    /// Number of bits to allacote for a single counter
    /// For width `w` bits, max frequency estimate will be `2^w - 1`
    ///
    /// Lower the width, smaller the sizes.
    ///
    /// Small values may affect the ranking of documents
    #[clap(short = 'w', long, default_value = "4")]
    counter_width: u32,
}
fn main() -> std::io::Result<()> {
    // Parse CLI options
    let opts = Opts::parse();

    let file = File::open(opts.tokens_file)?;
    let false_positive_rate = opts.false_positive_rate;
    let width = opts.counter_width;

    // Read file
    let buf_reader = BufReader::new(file);
    let tokens_json: Vec<Post> = serde_json::from_reader(buf_reader)?;

    // Create search index with base2p15 encoding
    let search_index: Vec<SearchItem> = tokens_json
        .into_iter()
        .map(|document| {
            let term_frequency = naive::tokenize(&document.body);
            let sbf = SpectralBloomFilter::new(&term_frequency, false_positive_rate, width);
            let encoded = base2p15::encode(&sbf.as_bit_string());

            SearchItem {
                url: document.url,
                title: document.title,
                sbf_base2p15: encoded,
                size: sbf.sbf.len() as u32,
                width: sbf.width,
                n_hash_functions: sbf.n_hash_functions,
            }
        })
        .collect();

    // Write to file using template
    // Instead of template engine, use string replace as hack
    let j = serde_json::to_string(&search_index)?;
    let js_template = include_str!("assets/static_website_search.js");
    let js_code = js_template.replace("UNIQUE_SEARCH_INDEX_PLACEHOLDER", j.as_str());
    std::fs::write("static_website_search.js", js_code)?;

    let demo_html = include_str!("assets/demo.html");
    std::fs::write("demo.html", demo_html)
}
