mod codelib;
mod parser;
mod relation_solver;
mod repo_collector;

use std::fs;

use codelib::CodeInfo;
pub use codelib::{Article, Collection, Commit, Lang, SourceSets};

use parser::{parse_code_info, parse_document};
use relation_solver::{solve_tested_by, CodeNode};
pub use repo_collector::gather_collection;

#[derive(Debug, Default)]
pub struct GatherResult {
    pub articles: Vec<Article>,
    // Files without any document block (path)
    pub skipped: Vec<String>,
    // (path, message)
    pub warnings: Vec<(String, String)>,
    // Files with an invalid document (path, message)
    pub errors: Vec<(String, String)>,
}

struct SourceFile {
    path: String,
    content: String,
    code_info: CodeInfo,
}

fn read_source_files(
    collection: &Collection,
    paths: &[String],
    lang: Lang,
) -> Result<Vec<SourceFile>, String> {
    paths
        .iter()
        .map(|path| {
            let total_path = collection.complete_path_str(path);
            let content =
                fs::read_to_string(&total_path).map_err(|e| format!("{}: {}", total_path, e))?;
            let code_info = parse_code_info(&content, lang);
            Ok(SourceFile {
                path: path.clone(),
                content,
                code_info,
            })
        })
        .collect()
}

fn to_code_nodes(files: &[SourceFile]) -> Vec<CodeNode<'_>> {
    files
        .iter()
        .map(|f| CodeNode {
            path: &f.path,
            code_info: &f.code_info,
        })
        .collect()
}

// Returns Err only if files cannot be read. Invalid documents are reported in GatherResult::errors.
pub fn complete_articles(collection: &Collection) -> Result<GatherResult, String> {
    let mut result = GatherResult::default();
    for source_sets in &collection.source_sets {
        let lang = source_sets.lang;
        let srcs = read_source_files(collection, &source_sets.src_paths, lang)?;
        let tests = read_source_files(collection, &source_sets.test_paths, lang)?;
        let mut tested_by = solve_tested_by(&to_code_nodes(&srcs), &to_code_nodes(&tests));

        for src in srcs {
            let commits = vec![]; // TODO:
            let src_tested_by = tested_by.remove(&src.path).unwrap_or_default();
            match parse_document(&src.content, src.path.clone(), lang, commits, src_tested_by) {
                Ok(Some(parsed)) => {
                    result.articles.push(parsed.article);
                    result
                        .warnings
                        .extend(parsed.warnings.into_iter().map(|w| (src.path.clone(), w)));
                }
                Ok(None) => result.skipped.push(src.path),
                Err(err) => result.errors.push((src.path, err)),
            }
        }
    }
    Ok(result)
}
