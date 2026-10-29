use crate::codelib::Commit;
use crate::{Article, CodeInfo, Lang};
use std::collections::BTreeMap;
use std::sync::LazyLock;

#[cfg(test)]
mod parser_snapshot_unittest;
#[cfg(test)]
mod unittest;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum BlockAnchor {
    Code,
    Article,
}

impl BlockAnchor {
    fn as_str(&self) -> &'static str {
        match self {
            BlockAnchor::Code => "CODE",
            BlockAnchor::Article => "DOC",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "CODE" => Some(BlockAnchor::Code),
            "DOC" => Some(BlockAnchor::Article),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SectionAnchor {
    Title,
    Overview,
    Usage,
    Require,
    Verified,
    References,
    Words,
    Tags,
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
            "tags" => Some(SectionAnchor::Tags),
            _ => None,
        }
    }
}

// Section being parsed. Unknown sections are parsed and discarded.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParsingSection {
    Known(SectionAnchor),
    Unknown,
}

#[derive(Debug)]
pub struct ParsedArticle {
    pub article: Article,
    pub warnings: Vec<String>,
}

struct ParserInternalState {
    block_anchor: Option<BlockAnchor>,
    has_block_anchor: bool,
    section_anchor: Option<ParsingSection>,
    parsing_text: String,
    collected_sections: BTreeMap<SectionAnchor, String>,
    collected_code: String,
    section_codeblock: bool,
    // Line number of the first known section anchor found outside of DOC blocks
    stray_section_line: Option<usize>,
    warnings: Vec<String>,
}

impl ParserInternalState {
    fn new() -> Self {
        Self {
            block_anchor: None,
            has_block_anchor: false,
            section_anchor: None,
            parsing_text: String::new(),
            collected_sections: BTreeMap::new(),
            collected_code: String::new(),
            section_codeblock: false,
            stray_section_line: None,
            warnings: Vec::new(),
        }
    }

    fn finish_anchor(&mut self) {
        if let Some(ParsingSection::Known(section)) = self.section_anchor.take() {
            let text = self.parsing_text.trim();
            // Duplicated sections are concatenated not to lose either of them
            let collected = self.collected_sections.entry(section).or_default();
            if !collected.is_empty() && !text.is_empty() {
                collected.push('\n');
            }
            collected.push_str(text);
        }
        self.parsing_text.clear();
    }

    fn parse_block_anchor(&mut self, anchor_str: &str, line_no: usize) -> Result<(), String> {
        let (begin, kind) = if let Some(kind) = anchor_str.strip_prefix("BEGIN ") {
            (true, kind)
        } else if let Some(kind) = anchor_str.strip_prefix("END ") {
            (false, kind)
        } else {
            return Err(format!(
                "line {}: Unknown block anchor: {}",
                line_no, anchor_str
            ));
        };
        let Some(block) = BlockAnchor::from_str(kind.trim()) else {
            return Err(format!(
                "line {}: Unknown block anchor: {}",
                line_no, anchor_str
            ));
        };
        self.has_block_anchor = true;

        if begin {
            if let Some(current) = self.block_anchor {
                return Err(format!(
                    "line {}: Nested block anchor is not allowed: {} in {} block",
                    line_no,
                    anchor_str,
                    current.as_str()
                ));
            }
            self.block_anchor = Some(block);
        } else {
            match self.block_anchor {
                Some(current) if current == block => {}
                Some(current) => {
                    return Err(format!(
                        "line {}: {} does not match the opened {} block",
                        line_no,
                        anchor_str,
                        current.as_str()
                    ))
                }
                None => {
                    return Err(format!(
                        "line {}: {} without BEGIN {}",
                        line_no,
                        anchor_str,
                        block.as_str()
                    ))
                }
            }
            if self.section_codeblock {
                return Err(format!("line {}: Code fence (```) is not closed", line_no));
            }
            self.finish_anchor();
            self.block_anchor = None;
        }
        Ok(())
    }

    fn start_section(&mut self, anchor: &str, line_no: usize) {
        self.finish_anchor();
        let section = match SectionAnchor::from_str(anchor) {
            Some(section) => {
                if self.collected_sections.contains_key(&section) {
                    self.warnings
                        .push(format!("line {}: Duplicated section: %{}", line_no, anchor));
                }
                ParsingSection::Known(section)
            }
            None => {
                self.warnings
                    .push(format!("line {}: Unknown section: %{}", line_no, anchor));
                ParsingSection::Unknown
            }
        };
        self.section_anchor = Some(section);
    }

    fn parse_line(&mut self, line: &str, line_no: usize) -> Result<(), String> {
        let comment = match_line_as_comment(line);

        if let Some(anchor_str) = comment.and_then(match_block_anchor) {
            return self.parse_block_anchor(anchor_str, line_no);
        }

        match self.block_anchor {
            // Code block: everything except block anchors is code
            Some(BlockAnchor::Code) => {
                self.collected_code.push_str(line);
                self.collected_code.push('\n');
            }
            Some(BlockAnchor::Article) => {
                if self.section_codeblock {
                    if comment.is_some_and(|c| c.starts_with("```")) {
                        self.section_codeblock = false;
                    } else {
                        self.parsing_text.push_str(line);
                        self.parsing_text.push('\n');
                    }
                } else if let Some(comment) = comment {
                    if let Some(anchor) = match_doc_anchor(comment) {
                        self.start_section(anchor, line_no);
                    } else if comment.starts_with("```") {
                        self.section_codeblock = true;
                    } else {
                        self.parsing_text.push_str(comment);
                        self.parsing_text.push('\n');
                    }
                }
            }
            None => {
                // Outside of blocks: ignore, but remember a section anchor (e.g. typo of %=BEGIN DOC)
                let is_section_anchor = comment
                    .and_then(match_doc_anchor)
                    .is_some_and(|anchor| SectionAnchor::from_str(anchor).is_some());
                if is_section_anchor && self.stray_section_line.is_none() {
                    self.stray_section_line = Some(line_no);
                }
            }
        }
        Ok(())
    }

    fn generate_article(
        self,
        path: String,
        lang: Lang,
        commits: Vec<Commit>,
        tested_by: Vec<String>,
    ) -> Result<ParsedArticle, String> {
        if let Some(block) = self.block_anchor {
            return Err(format!("{} block is not closed", block.as_str()));
        }

        if self.collected_code.trim().is_empty() {
            return Err("Code block is empty".to_string());
        }

        let mut sections = self.collected_sections;
        let mut take = |section: SectionAnchor| sections.remove(&section);

        let Some(title) = take(SectionAnchor::Title) else {
            return Err("Title is required".to_string());
        };

        let article = Article {
            title,
            overview: take(SectionAnchor::Overview).unwrap_or_default(),
            code: self.collected_code.trim().to_string(),
            lang,
            path,
            require: take(SectionAnchor::Require).unwrap_or_default(),
            // One reference per line (a reference may contain spaces)
            references: take(SectionAnchor::References)
                .map(|s| {
                    s.lines()
                        .map(|l| l.trim())
                        .filter(|l| !l.is_empty())
                        .map(|l| l.to_string())
                        .collect()
                })
                .unwrap_or_default(),
            words: take(SectionAnchor::Words)
                .map(|s| {
                    s.split([',', ' ', '\n', '\t'])
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default(),
            verified: take(SectionAnchor::Verified)
                .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            commits,
            tested_by,
        };
        Ok(ParsedArticle {
            article,
            warnings: self.warnings,
        })
    }
}

static INCLUDE_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    // unwrap is ok because the pattern is always valid
    regex::Regex::new(r#"^#include\s*"([^"]*)""#).unwrap()
});

fn parse_code_info_cpp(content: &str) -> CodeInfo {
    let filepath_dependencies = content
        .lines()
        // Extract the path from the `#include` directive. <path> will be ignored.
        .filter_map(|line| INCLUDE_REGEX.captures(line))
        // TODO: normalize path?
        .map(|captures| format!("/{}", &captures[1]))
        .collect();
    CodeInfo {
        filepath_dependencies,
    }
}

// ----------------------------------------------------------------------------

// Returns Ok(None) if the content has no block anchor (= not a document).
pub fn parse_document(
    content: &str,
    article_path: String,
    lang: Lang,
    commits: Vec<Commit>,
    tested_by: Vec<String>,
) -> Result<Option<ParsedArticle>, String> {
    let mut parser_state = ParserInternalState::new();

    for (i, line) in content.lines().enumerate() {
        parser_state.parse_line(line, i + 1)?;
    }
    parser_state.finish_anchor();

    if let Some(line_no) = parser_state.stray_section_line {
        let message = format!(
            "line {}: Section anchor outside of DOC block (missing %=BEGIN DOC?)",
            line_no
        );
        if !parser_state.has_block_anchor {
            return Err(message);
        }
        parser_state.warnings.push(message);
    }
    if !parser_state.has_block_anchor {
        return Ok(None);
    }
    parser_state
        .generate_article(article_path, lang, commits, tested_by)
        .map(Some)
}

pub fn parse_code_info(content: &str, lang: Lang) -> CodeInfo {
    match lang {
        Lang::Cpp => parse_code_info_cpp(content),
        _ => CodeInfo::default(),
    }
}
