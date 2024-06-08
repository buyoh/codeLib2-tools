mod codelib;
mod parser;
mod relation_solver;
mod repo_collector;

use std::fs::File;

pub use codelib::Article;

// TODO: remove several pub
pub use parser::parse_code_info_from_file;
pub use parser::parse_document_from_file;
pub use relation_solver::solve_relation;
pub use repo_collector::gather_collection;
pub use repo_collector::gather_commit_info;

pub fn complete_articles(base_path: &str) -> Vec<Article> {
    let collection = gather_collection(&base_path).unwrap();

    let mut articles = Vec::new();
    for (src_paths, lang) in collection.src_paths.iter().zip(collection.langs.iter()) {
        for src_path in src_paths {
            let total_path = collection.complete_path_str(src_path);
            let file = File::open(&total_path).expect("Failed to open file");
            let commits = vec![]; // TODO:
            let tested_by = vec![]; // TODO:
            let article =
                parse_document_from_file(file, src_path.clone(), lang.clone(), commits, tested_by);
            articles.push(article.unwrap());
        }
    }
    articles
}
