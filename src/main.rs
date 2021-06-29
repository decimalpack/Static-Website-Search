use clap::Clap;
use serde::{Deserialize, Serialize};
use static_website_search::compressor::base2p15;
use static_website_search::estimator::spectral_bloom_filter::SpectralBloomFilter;
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

#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
struct Opts {
    /// The tokens.json file from which to read.
    /// The file has the following structure:
    ///
    /// ```json
    ///  [
    ///      {
    ///          "document_link":"doc1_link"
    ///          "term_frequency": {
    ///                  // word as key and frequency as value
    ///                  "word1":1,
    ///                  "word2":2,
    ///           }
    ///      },
    ///      {
    ///          "document_link":"doc2_link"
    ///          "term_frequency": {
    ///                  "word1":1,
    ///                  "word2":2,
    ///           }
    ///      }
    ///      // And so on
    ///  ]
    /// ```json
    #[clap(short, long)]
    tokens_file: String,

    /// Name of the output file
    #[clap(short, long, default_value = "search.html")]
    output_file: String,

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
    let tokens_json: Vec<Document> = serde_json::from_reader(buf_reader)?;

    // Create search index with base2p15 encoding
    let search_index: Vec<SearchItem> = tokens_json
        .into_iter()
        .map(|document| {
            let sbf =
                SpectralBloomFilter::new(&document.term_frequency, false_positive_rate, width);
            let encoded = base2p15::encode(&sbf.as_bit_string());

            SearchItem {
                sbf_base2p15: encoded,
                width: sbf.width,
                n_hash_functions: sbf.n_hash_functions,
                document_link: document.document_link,
            }
        })
        .collect();

    // Write to file using template
    // Instead of template engine, use string replace as hack
    let j = serde_json::to_string(&search_index)?;
    let template = include_str!("assets/search_template.html");
    let search_page = template.replace("UNIQUE_SEARCH_INDEX_PLACEHOLDER", j.as_str());
    std::fs::write(opts.output_file, search_page)
}
