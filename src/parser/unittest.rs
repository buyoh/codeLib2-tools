use super::{parse_code_info, parse_document, ParsedArticle};
use crate::Lang;

fn parse(content: &str) -> Result<Option<ParsedArticle>, String> {
    parse_document(
        content,
        "/src/cpp/a.hpp".to_string(),
        Lang::Cpp,
        vec![],
        vec![],
    )
}

fn parse_ok(content: &str) -> ParsedArticle {
    parse(content)
        .expect("parse must succeed")
        .expect("must be a document")
}

const MINIMAL: &str = "\
// %=BEGIN DOC
// %title
// T
// %=END DOC
// %=BEGIN CODE
int f();
// %=END CODE
";

#[test]
fn test_minimal() {
    let parsed = parse_ok(MINIMAL);
    assert_eq!(parsed.article.title, "T");
    assert_eq!(parsed.article.code, "int f();");
    assert!(parsed.warnings.is_empty());
}

#[test]
fn test_no_document() {
    assert!(parse("").unwrap().is_none());
    assert!(parse("int f();\n// comment\n// 100%\n// %unknown\n")
        .unwrap()
        .is_none());
}

#[test]
fn test_section_anchor_outside_of_doc_block() {
    // Typo of %=BEGIN DOC
    let err = parse("// %BEGIN DOC\n// %title\n// T\nint f();\n").unwrap_err();
    assert!(err.contains("line 2"), "{}", err);
    assert!(err.contains("outside of DOC block"), "{}", err);

    let content = format!("// %title\n{}", MINIMAL);
    let parsed = parse_ok(&content);
    assert_eq!(parsed.warnings.len(), 1, "{:?}", parsed.warnings);
    assert!(parsed.warnings[0].contains("outside of DOC block"));
}

#[test]
fn test_error_messages_use_anchor_names() {
    let err = parse("// %=END DOC\n").unwrap_err();
    assert_eq!(err, "line 1: END DOC without BEGIN DOC");
    let err = parse("// %=BEGIN CODE\n// %=END DOC\n").unwrap_err();
    assert_eq!(err, "line 2: END DOC does not match the opened CODE block");
    let err = parse("// %=BEGIN DOC\n").unwrap_err();
    assert_eq!(err, "DOC block is not closed");
}

#[test]
fn test_multiple_blocks_are_concatenated() {
    let content = format!(
        "{}// %=BEGIN DOC\n// %overview\n// O\n// %=END DOC\n// %=BEGIN CODE\nint g();\n// %=END CODE\n",
        MINIMAL
    );
    let parsed = parse_ok(&content);
    assert_eq!(parsed.article.code, "int f();\nint g();");
    assert_eq!(parsed.article.overview, "O");
}

#[test]
fn test_nested_block_is_error() {
    let content = "// %=BEGIN CODE\n// %=BEGIN CODE\nint f();\n// %=END CODE\n";
    assert!(parse(content).unwrap_err().contains("Nested"));
    let content = "// %=BEGIN DOC\n// %=BEGIN CODE\n";
    assert!(parse(content).unwrap_err().contains("Nested"));
}

#[test]
fn test_mismatched_end_is_error() {
    let content = "// %=BEGIN CODE\nint f();\n// %=END DOC\n";
    let err = parse(content).unwrap_err();
    assert!(err.contains("line 3"), "{}", err);
    assert!(parse("// %=END DOC\n").is_err());
}

#[test]
fn test_unknown_block_anchor_is_error() {
    assert!(parse("// %=BEGIN FOO\n").is_err());
    assert!(parse("// %=FOO\n").is_err());
}

#[test]
fn test_unclosed_block_is_error() {
    let content = "// %=BEGIN DOC\n// %title\n// T\n// %=END DOC\n// %=BEGIN CODE\nint f();\n";
    assert!(parse(content).unwrap_err().contains("not closed"));
}

#[test]
fn test_unclosed_code_fence_is_error() {
    let content = "// %=BEGIN DOC\n// %title\n// T\n// ```\n// %=END DOC\n";
    assert!(parse(content).unwrap_err().contains("fence"));
}

#[test]
fn test_title_and_code_are_required() {
    let content = "// %=BEGIN DOC\n// %overview\n// O\n// %=END DOC\n// %=BEGIN CODE\nint f();\n// %=END CODE\n";
    assert!(parse(content).unwrap_err().contains("Title"));
    let content =
        "// %=BEGIN DOC\n// %title\n// T\n// %=END DOC\n// %=BEGIN CODE\n\n// %=END CODE\n";
    assert!(parse(content).unwrap_err().contains("Code block is empty"));
}

#[test]
fn test_unknown_and_duplicated_section_are_warned() {
    let content = MINIMAL.replace(
        "// %=END DOC",
        "// %tag\n// nolint\n// %title\n// T2\n// %tags\n// nolint\n// %=END DOC",
    );
    let parsed = parse_ok(&content);
    assert_eq!(parsed.warnings.len(), 2, "{:?}", parsed.warnings);
    assert!(parsed.warnings[0].contains("Unknown section: %tag"));
    assert!(parsed.warnings[1].contains("Duplicated section: %title"));
    // Content of the unknown section is discarded, duplicated sections are concatenated
    assert_eq!(parsed.article.title, "T\nT2");
}

#[test]
fn test_duplicated_empty_section_does_not_lose_value() {
    let content = MINIMAL.replace(
        "// %=END DOC",
        "// %require\n// ```\n#include <vector>\n// ```\n// %words\n// w\n// %require\n// %=END DOC",
    );
    let parsed = parse_ok(&content);
    assert_eq!(parsed.article.require, "#include <vector>");
}

#[test]
fn test_code_block_keeps_comment_lines() {
    let content = MINIMAL.replace("int f();", "// %title\n// ```\n# comment\nint f();");
    let parsed = parse_ok(&content);
    assert_eq!(
        parsed.article.code,
        "// %title\n// ```\n# comment\nint f();"
    );
    assert_eq!(parsed.article.title, "T");
}

#[test]
fn test_code_fence_in_section() {
    let content = MINIMAL.replace(
        "// %=END DOC",
        "// %require\n// ```\n#include <vector>\n// %title\n// ```\n// %=END DOC",
    );
    let parsed = parse_ok(&content);
    assert_eq!(parsed.article.require, "#include <vector>\n// %title");
    assert_eq!(parsed.article.title, "T");
}

#[test]
fn test_words_have_no_empty_string() {
    let content = MINIMAL.replace("// %=END DOC", "// %words\n// a, b\n// c\td,\n// %=END DOC");
    let parsed = parse_ok(&content);
    assert_eq!(parsed.article.words, vec!["a", "b", "c", "d"]);
}

#[test]
fn test_references_are_split_by_line() {
    let content = MINIMAL.replace(
        "// %=END DOC",
        "// %references\n// https://example.com/a\n//\n// Book title, thanks.\n// %=END DOC",
    );
    let parsed = parse_ok(&content);
    assert_eq!(
        parsed.article.references,
        vec!["https://example.com/a", "Book title, thanks."]
    );
}

#[test]
fn test_parse_code_info_cpp() {
    let content = "#include <vector>\n#include \"src/cpp/a.hpp\" // \"b\"\n#include\"src/cpp/b.hpp\"\n  #include \"src/cpp/c.hpp\"\n";
    let code_info = parse_code_info(content, Lang::Cpp);
    assert_eq!(
        code_info.filepath_dependencies,
        vec!["/src/cpp/a.hpp", "/src/cpp/b.hpp"]
    );
    assert!(parse_code_info(content, Lang::Rust)
        .filepath_dependencies
        .is_empty());
}
