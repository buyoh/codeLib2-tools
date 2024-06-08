use crate::{CodeInfo, CodeInfoSets, Collection, SourceSets};

#[cfg(test)]
use super::solve_relation;

#[test]
fn test() {
    let collection = Collection {
        base_path: "test".to_string(),
        source_sets: vec![SourceSets {
            lang: "cpp".to_string(),
            src_paths: vec![
                "src/a/x.hpp".to_string(),
                "src/a/y.hpp".to_string(),
                "src/b/x.hpp".to_string(),
                "src/b/y.hpp".to_string(),
            ],
            test_paths: vec!["test/1.cpp".to_string(), "test/2.cpp".to_string()],
        }],
    };
    let code_info_sets = vec![CodeInfoSets {
        lang: "cpp".to_string(),
        src_code_infos: vec![
            CodeInfo {
                filepath_dependencies: vec![],
            },
            CodeInfo {
                filepath_dependencies: vec!["src/a/x.hpp".to_string()],
            },
            CodeInfo {
                filepath_dependencies: vec!["src/a/x.hpp".to_string()],
            },
            CodeInfo {
                filepath_dependencies: vec!["src/a/x.hpp".to_string(), "src/b/x.hpp".to_string()],
            },
        ],
        test_code_infos: vec![
            CodeInfo {
                filepath_dependencies: vec!["src/a/x.hpp".to_string()],
            },
            CodeInfo {
                filepath_dependencies: vec!["src/b/y.hpp".to_string()],
            },
        ],
    }];
    let relations = solve_relation(&collection, &code_info_sets).unwrap();
    assert_eq!(relations.source_relations.len(), 1);

    let source_relation = &relations.source_relations[0];
    assert_eq!(source_relation.len(), 4);
    assert_eq!(source_relation[1].tested_by, vec![] as Vec<usize>);
    assert_eq!(source_relation[2].tested_by, vec![1]);
    assert_eq!(source_relation[3].tested_by, vec![1]);
    assert_eq!(source_relation[0].tested_by, vec![0, 1]);
}
