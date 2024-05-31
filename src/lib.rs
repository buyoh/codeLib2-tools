mod codelib;
mod parser;

use chrono::{DateTime, FixedOffset};
pub use codelib::Article;

pub use parser::parse_document_from_file;

pub fn from_json_array(json_data: &str) -> Vec<Article> {
    serde_json::from_str(json_data).unwrap()
}
