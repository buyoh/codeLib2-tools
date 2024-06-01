mod codelib;
mod parser;

use chrono::{DateTime, FixedOffset};
pub use codelib::Article;

pub use parser::parse_document_from_file;

