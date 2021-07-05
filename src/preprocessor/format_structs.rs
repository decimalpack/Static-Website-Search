use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Deserialize, Debug)]
pub struct Post {
    pub title: String,
    pub url: String,
    pub body: String,
}

pub struct PostFreq {
    pub title: String,
    pub url: String,
    pub term_frequency: HashMap<String, u32>,
}

#[derive(Debug, Serialize)]
pub struct SearchItem {
    pub url: String,
    pub title: String,
    pub sbf_base2p15: String,
    pub width: u32,
    pub size: u32,
    pub n_hash_functions: u32,
}
