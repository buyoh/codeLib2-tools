mod codelib;
mod parser;
mod relation_solver;
mod repo_collector;

use std::fs::File;

use clap::Error;
// TODO: remove several pub
pub use codelib::{Article, CodeInfo, CodeInfoSets, Collection, Commit, SourceSets};

use parser::parse_code_info_from_file;
use parser::parse_document_from_file;
use relation_solver::solve_relation;
pub use repo_collector::gather_collection;
// use repo_collector::gather_commit_info;

fn collect_code_infos(collection: &Collection) -> Vec<CodeInfoSets> {
    collection
        .source_sets
        .iter()
        .map(|source_sets| {
            let src_code_infos = source_sets
                .src_paths
                .iter()
                .map(|src_path| {
                    let total_path = collection.complete_path_str(src_path);
                    let file = File::open(&total_path).expect("Failed to open file");
                    let code_info = parse_code_info_from_file(file, source_sets.lang.clone())
                        .expect("Failed to parse code info");
                    code_info
                })
                .collect();

            let test_code_infos = source_sets
                .test_paths
                .iter()
                .map(|test_path| {
                    let total_path = collection.complete_path_str(test_path);
                    let file = File::open(&total_path).expect("Failed to open file");
                    let code_info = parse_code_info_from_file(file, source_sets.lang.clone())
                        .expect("Failed to parse code info");
                    code_info
                })
                .collect();

            CodeInfoSets {
                lang: source_sets.lang.clone(),
                src_code_infos,
                test_code_infos,
            }
        })
        .collect()
}

pub fn complete_articles(collection : &Collection) -> Result<Vec<Article>, String> {
    let code_info_sets_vec = collect_code_infos(&collection);
    let relations = match solve_relation(&collection, &code_info_sets_vec) {
        Ok(relations) => relations,
        Err(err) => return Err(err),
    };

    let mut articles = Vec::new();
    for (i, source_sets) in collection.source_sets.iter().enumerate() {
        let lang = &source_sets.lang;
        let rels = &relations.source_relations[i];
        let src_paths = &source_sets.src_paths;
        let test_paths = &source_sets.test_paths;

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
    Ok(articles)
}
