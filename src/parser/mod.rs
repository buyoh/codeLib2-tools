use crate::Article;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn match_line_as_comment(line: &str) -> Option<&str> {
    // 先頭が # で始まる行なら、先頭の # と後続するスペースを取り除いた文字列を返す
    if line.starts_with("#") {
        Some(line.trim_start_matches("#").trim_start())
    } else if line.starts_with("//") {
        Some(line.trim_start_matches("//").trim_start())
    } else {
        None
    }
}

fn match_block_anchor(comment: &str) -> Option<&str> {
    if comment.starts_with("%=") {
        Some(comment.trim_start_matches("%=").trim())
    } else {
        None
    }
}

fn match_doc_anchor(comment: &str) -> Option<&str> {
    if comment.starts_with("%") {
        Some(comment.trim_start_matches("%").trim())
    } else {
        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BlockAnchor {
    Code,
    Article,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SectionAnchor {
    Title,
    Overview,
    Usage,
    Require,
    Verified,
    References,
    Words,
    Unknown(String),
}

// Contert str to optional SectionAnchor
impl SectionAnchor {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "title" => Some(SectionAnchor::Title),
            "overview" => Some(SectionAnchor::Overview),
            "usage" => Some(SectionAnchor::Usage),
            "require" => Some(SectionAnchor::Require),
            "verified" => Some(SectionAnchor::Verified),
            "references" => Some(SectionAnchor::References),
            "words" => Some(SectionAnchor::Words),
            _ => None,
        }
    }
}

struct ParserInternalState {
    block_anchor: Option<BlockAnchor>,
    section_anchor: Option<SectionAnchor>,
    parsing_text: String,
    collected_sections: BTreeMap<SectionAnchor, String>,
    collected_code: String,
}

impl ParserInternalState {
    fn new() -> Self {
        Self {
            block_anchor: None,
            section_anchor: None,
            parsing_text: String::new(),
            collected_sections: BTreeMap::new(),
            collected_code: String::new(),
        }
    }

    fn finish_anchor(&mut self) {
        if let Some(section) = std::mem::replace(&mut self.section_anchor, None) {
            self.collected_sections.insert(
                section,
                std::mem::replace(&mut self.parsing_text, String::new()),
            );
        }
        self.section_anchor = None;
        self.parsing_text.clear();
    }

    fn parse_line(&mut self, line: &str) -> Result<(), String> {
        if let Some(comment) = match_line_as_comment(line) {
            if let Some(anchor_str) = match_block_anchor(comment) {
                match anchor_str {
                    "BEGIN DOC" => {
                        if let Some(_) = self.block_anchor {
                            return Err("Nested block anchor is not allowed".to_string());
                        }
                        self.block_anchor = Some(BlockAnchor::Article);
                    }
                    "BEGIN CODE" => {
                        self.block_anchor = Some(BlockAnchor::Code);
                    }
                    "END DOC" => {
                        self.finish_anchor();
                        self.block_anchor = None;
                    }
                    "END CODE" => {
                        self.finish_anchor();
                        self.block_anchor = None;
                    }
                    _ => {
                        return Err(format!("Unknown block anchor: {}", anchor_str));
                    }
                }
            } else if let Some(anchor) = match_doc_anchor(comment) {
                // Article ブロックのみ有効
                if let Some(BlockAnchor::Article) = self.block_anchor {
                    if let Some(section) = SectionAnchor::from_str(anchor) {
                        self.finish_anchor();
                        self.section_anchor = Some(section);
                    } else {
                        self.finish_anchor();
                        self.section_anchor = Some(SectionAnchor::Unknown(anchor.to_string()));
                    }
                }
            } else {
                // Add to text
                self.parsing_text.push_str(comment);
            }
        } else {
            // Add code to text
            self.collected_code.push_str(line);
        }
        Ok(())
    }

    fn generate_article(self) -> Article {
        Article {
            title: self
                .collected_sections
                .get(&SectionAnchor::Title)
                .unwrap_or(&String::new())
                .clone(),
            overview: self
                .collected_sections
                .get(&SectionAnchor::Overview)
                .unwrap_or(&String::new())
                .clone(),
            code: self.collected_code,
            lang: "rust".to_string(),
            path: "unknown".to_string(),
            require: self
                .collected_sections
                .get(&SectionAnchor::Require)
                .map(|s| s.clone()),
            references: self
                .collected_sections
                .get(&SectionAnchor::References)
                .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or(Vec::new()),
            words: self
                .collected_sections
                .get(&SectionAnchor::Words)
                .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or(Vec::new()),
            verified: self
                .collected_sections
                .get(&SectionAnchor::Verified)
                .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or(Vec::new()),
            commits: Vec::new(),
            tested_by: Vec::new(),
        }
    }

}

pub fn parse_document_from_file(path: &str) -> Result<Article, String> {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut parser_state = ParserInternalState::new();

    for may_line in reader.lines() {
        if let Ok(line) = may_line {
            if let Err(err) = parser_state.parse_line(&line) {
                return Err(err);
            }
        }
    }
    parser_state.finish_anchor();

    Ok(parser_state.generate_article())
}
