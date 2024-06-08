use crate::codelib::Commit;
use crate::{Article, CodeInfo};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[cfg(test)]
mod parser_snapshot_unittest;

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
    section_codeblock: bool,
}

impl ParserInternalState {
    fn new() -> Self {
        Self {
            block_anchor: None,
            section_anchor: None,
            parsing_text: String::new(),
            collected_sections: BTreeMap::new(),
            collected_code: String::new(),
            section_codeblock: false,
        }
    }

    fn finish_anchor(&mut self) {
        if let Some(section) = std::mem::replace(&mut self.section_anchor, None) {
            let trimmed_text = self.parsing_text.trim();
            self.collected_sections.insert(
                section,
                trimmed_text.to_string(), // no way to avoid clone here
            );
            self.parsing_text = String::new();
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
            } else if comment.starts_with("```") {
                if self.section_codeblock {
                    self.section_codeblock = false;
                } else {
                    self.section_codeblock = true;
                }
            } else {
                if self.section_codeblock {
                    self.parsing_text.push_str(line);
                    self.parsing_text.push_str("\n");
                } else {
                    self.parsing_text.push_str(comment);
                    self.parsing_text.push_str("\n");
                }
            }
        } else {
            if let Some(BlockAnchor::Code) = self.block_anchor {
                self.collected_code.push_str(line);
                self.collected_code.push_str("\n");
            } else if self.section_codeblock {
                self.parsing_text.push_str(line);
                self.parsing_text.push_str("\n");
            } else {
                // ignore
            }
        }
        Ok(())
    }

    fn generate_article(
        self,
        path: String,
        lang: String,
        commits: Vec<Commit>,
        tested_by: Vec<String>,
    ) -> Result<Article, String> {
        if let Some(_) = self.block_anchor {
            return Err("Block anchor is not closed".to_string());
        }

        if self.collected_code.is_empty() {
            return Err("Code block is empty".to_string());
        }

        if self.collected_sections.get(&SectionAnchor::Title).is_none() {
            return Err("Title is required".to_string());
        }

        Ok(Article {
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
            code: self.collected_code.trim().to_string(),
            lang,
            path,
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
                .map(|s| {
                    s.split([',', ' ', '\n', '\t'])
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or(Vec::new()),
            verified: self
                .collected_sections
                .get(&SectionAnchor::Verified)
                .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or(Vec::new()),
            commits,
            tested_by,
        })
    }
}

fn parse_code_info_from_file_cpp(file: File) -> Result<CodeInfo, String> {
    let reader = BufReader::new(file);
    let mut filepath_dependencies = Vec::new();
    for may_line in reader.lines() {
        let line = match may_line {
            Ok(line) => line,
            Err(_) => continue,
        };
        // Is this line a `#include` directive?
        if line.starts_with("#include") {
            // Extract the path from the `#include` directive by regexp
            // <path> will be ignored.
            let re = regex::Regex::new(r#"#include\s*["](.*)["]"#).unwrap();
            let captures = re.captures(&line);
            if let Some(captures) = captures {
                let path = captures.get(1).unwrap().as_str();
                // TODO: normalize path?
                filepath_dependencies.push(format!("/{}", path));
            }
        }
    }
    Ok(CodeInfo {
        filepath_dependencies,
    })
}

// ----------------------------------------------------------------------------

pub fn parse_document_from_file(
    file: File,
    article_path: String,
    lang: String,
    commits: Vec<Commit>,
    tested_by: Vec<String>,
) -> Result<Article, String> {
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

    parser_state.generate_article(article_path, lang, commits, tested_by)
}

pub fn parse_code_info_from_file(file: File, lang: String) -> Result<CodeInfo, String> {
    match lang.as_str() {
        "cpp" => parse_code_info_from_file_cpp(file),
        _ => Ok(CodeInfo {
            filepath_dependencies: Vec::new(),
        }),
    }
}
