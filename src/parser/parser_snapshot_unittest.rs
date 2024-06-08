use crate::codelib::Commit;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::File;

struct AdditionalInfo {
    lang: String,
    path: String,
    commits: Vec<Commit>,
    tested_by: Vec<String>,
}

fn read_additional_info(mut file: File) -> AdditionalInfo {
    let json: BTreeMap<String, Value> =
        serde_json::from_reader(&mut file).expect("Failed to parse JSON");

    let mut lang = None as Option<String>;
    let mut path = None as Option<String>;
    let mut commits = None as Option<Vec<Commit>>; // empty is ok
    let mut tested_by = None as Option<Vec<String>>; // empty is ok
    for (key, val) in json {
        match key.as_str() {
            "lang" => lang = Some(val.as_str().expect("lang must be a string").to_string()),
            "path" => path = Some(val.as_str().expect("path must be a string").to_string()),
            "commits" => {
                let mut commit_vec = Vec::new();
                for commit in val.as_array().expect("commits must be an array") {
                    let commit = commit.as_object().expect("commit must be an object");
                    let sha = commit
                        .get("sha")
                        .expect("sha must be a string")
                        .as_str()
                        .expect("sha must be a string")
                        .to_string();
                    let date = commit
                        .get("date")
                        .expect("date must be a string")
                        .as_str()
                        .expect("date must be a string")
                        .to_string();
                    let message = commit
                        .get("message")
                        .expect("message must be a string")
                        .as_str()
                        .expect("message must be a string")
                        .to_string();
                    commit_vec.push(Commit { sha, date, message });
                }
                commits = Some(commit_vec);
            }
            "tested_by" => {
                tested_by = Some(
                    val.as_array()
                        .expect("tested_by must be an array")
                        .iter()
                        .map(|s| s.as_str().expect("tested_by must be a string").to_string())
                        .collect(),
                )
            }
            _ => panic!("Unknown key: {}", key),
        }
    }
    AdditionalInfo {
        lang: lang.expect("lang is required"),
        path: path.expect("path is required"),
        commits: commits.expect("commits is required"),
        tested_by: tested_by.expect("tested_by is required"),
    }
}

fn test_parse_file(path: &str) {
    let path_without_ext = path.split('.').next().expect("path must have an extension");
    let path_additional = format!("{}.json", path_without_ext);
    let path_expected = format!("{}.out.json", path_without_ext);

    let additional_info =
        read_additional_info(File::open(path_additional).expect("Failed to open file"));

    let expected_json: Value =
        serde_json::from_reader(File::open(path_expected).expect("Failed to open file"))
            .expect("Failed to parse JSON");

    let parsed_article = crate::parser::parse_document_from_file(
        File::open(path).expect("Failed to open file"),
        additional_info.path,
        additional_info.lang,
        additional_info.commits,
        additional_info.tested_by,
    )
    .expect("Failed to parse article");

    let article_json = serde_json::to_value(&parsed_article).expect("Failed to serialize article");

    assert_eq!(article_json, expected_json);
}

#[test]
fn test() {
    test_parse_file("src/parser/unittest_resource/rmq_sparsetable.hpp");
}
