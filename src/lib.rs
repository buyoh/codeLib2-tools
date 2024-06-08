mod codelib;
mod parser;
mod relation_solver;
mod repo_collector;

use std::fs::File;

pub use codelib::Article;

// TODO: remove several pub
use parser::parse_code_info_from_file;
use parser::parse_document_from_file;
use relation_solver::solve_relation;
use repo_collector::gather_collection;
// use repo_collector::gather_commit_info;
use repo_collector::Collection;

fn collect_code_infos(
    collection: &Collection,
) -> (Vec<Vec<parser::CodeInfo>>, Vec<Vec<parser::CodeInfo>>) {
    let src_code_infos = collection
        .src_paths
        .iter()
        .zip(collection.langs.iter())
        .map(|(src_paths, lang)| {
            src_paths
                .iter()
                .map(|src_path| {
                    let total_path = collection.complete_path_str(src_path);
                    let file = File::open(&total_path).expect("Failed to open file");
                    let code_info = parse_code_info_from_file(file, lang.clone())
                        .expect("Failed to parse code info");
                    code_info
                })
                .collect()
        })
        .collect();

    let test_code_infos = collection
        .test_paths
        .iter()
        .zip(collection.langs.iter())
        .map(|(test_paths, lang)| {
            test_paths
                .iter()
                .map(|test_path| {
                    let total_path = collection.complete_path_str(test_path);
                    let file = File::open(&total_path).expect("Failed to open file");
                    let code_info = parse_code_info_from_file(file, lang.clone())
                        .expect("Failed to parse code info");
                    code_info
                })
                .collect()
        })
        .collect();

    (src_code_infos, test_code_infos)
}

pub fn complete_articles(base_path: &str) -> Vec<Article> {
    let collection = gather_collection(&base_path).unwrap();

    let (src_code_infos, test_code_infos) = collect_code_infos(&collection);
    let relations = solve_relation(&collection, &src_code_infos, &test_code_infos).unwrap();

    let mut articles = Vec::new();
    for (i, lang) in collection.langs.iter().enumerate() {
        let rels = &relations.source_relations[i];
        let src_paths = &collection.src_paths[i];
        let test_paths = &collection.test_paths[i];

        for (src_path, source_relation) in src_paths.iter().zip(rels.iter()) {
            let total_path = collection.complete_path_str(src_path);
            let file = File::open(&total_path).expect("Failed to open file");
            let commits = vec![]; // TODO:
            let tested_by = source_relation
                .tested_by
                .iter()
                .map(|i| test_paths[*i].clone())
                .collect();
            let article =
                parse_document_from_file(file, src_path.clone(), lang.clone(), commits, tested_by);
            
            articles.push(match article {
                Ok(article) => article,
                Err(err) => {
                    eprintln!("Failed to parse article: {}: {}", src_path, err);
                    continue;
                }
            });
        }
    }
    articles
}
