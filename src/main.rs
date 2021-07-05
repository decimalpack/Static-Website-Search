use clap::Clap;
use static_website_search::compressor::base2p15;
use static_website_search::estimator::spectral_bloom_filter::SpectralBloomFilter;
use static_website_search::preprocessor::format_structs::{Post, PostFreq, SearchItem};
use static_website_search::preprocessor::minimize_width::minimize_width;
use static_website_search::preprocessor::naive;
use std::fs::File;
use std::io::BufReader;

#[derive(Clap)]
#[clap(
    version = "0.1",
    about = "https://github.com/decimalpack/Static-Website-Search"
)]
struct Opts {
    /// The tokens.json file from which to read.
    #[clap(short, long)]
    tokens_file: String,

    /// The false positive rate, lower rate means higher sizes
    ///
    /// High values will introduce more false positives in search results
    #[clap(short, long, default_value = "0.01")]
    false_positive_rate: f32,
}

/// Create search item with base2p15 encoding
fn process_post(post: PostFreq, false_positive_rate: f32) -> SearchItem {
    // Calculate width as number of bits needed to represent max frequency
    let max_freq: u32 = post.term_frequency.iter().map(|(_, &f)| f).max().unwrap();
    let width = max_freq.next_power_of_two().trailing_zeros();
    let width = std::cmp::max(width, 1); // Avoid width = 0

    let sbf = SpectralBloomFilter::new(&post.term_frequency, false_positive_rate, width);
    let encoded = base2p15::encode(&sbf.as_bit_string());

    SearchItem {
        url: post.url,
        title: post.title,
        sbf_base2p15: encoded,
        size: sbf.sbf.len() as u32,
        width: sbf.width,
        n_hash_functions: sbf.n_hash_functions,
    }
}
fn main() -> std::io::Result<()> {
    // Parse CLI options
    let opts = Opts::parse();

    let file = File::open(opts.tokens_file)?;
    let false_positive_rate = opts.false_positive_rate;

    // Read file
    let buf_reader = BufReader::new(file);
    let tokens_json: Vec<Post> = serde_json::from_reader(buf_reader)?;

    let posts: Vec<PostFreq> = tokens_json
        .into_iter()
        .map(|post| PostFreq {
            title: post.title,
            url: post.url,
            term_frequency: naive::tokenize(&post.body),
        })
        .collect();

    let posts = minimize_width(&posts);

    let search_index: Vec<SearchItem> = posts
        .into_iter()
        .map(|post| process_post(post, false_positive_rate))
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
