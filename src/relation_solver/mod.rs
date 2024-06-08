use std::collections::{BTreeMap, BTreeSet};

use crate::{parser::CodeInfo, repo_collector::Collection};

#[cfg(test)]
mod unittest;

// TODO: not in relation_solver private?
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CodeIndex {
    Src(usize, usize),  // lang_idx, code_idx
    Test(usize, usize), // lang_idx, code_idx
}

struct RelationInternalSolver<'a> {
    collection: &'a Collection,
    src_code_infos: &'a Vec<Vec<CodeInfo>>,
    test_code_infos: &'a Vec<Vec<CodeInfo>>,

    // source_relations: Vec<SourceRelationSolving>,
    path_to_code_index: BTreeMap<String, CodeIndex>,
}

fn create_map_path_to_code_index(
    collection: &Collection,
    src_code_infos: &Vec<Vec<CodeInfo>>,
    test_code_infos: &Vec<Vec<CodeInfo>>,
) -> BTreeMap<String, CodeIndex> {
    let mut map = BTreeMap::new();
    for (lang_idx, lang_src_paths) in collection.src_paths.iter().enumerate() {
        for (code_idx, src_path) in lang_src_paths.iter().enumerate() {
            map.insert(src_path.clone(), CodeIndex::Src(lang_idx, code_idx));
        }
    }
    for (lang_idx, lang_test_paths) in collection.test_paths.iter().enumerate() {
        for (code_idx, test_path) in lang_test_paths.iter().enumerate() {
            map.insert(test_path.clone(), CodeIndex::Test(lang_idx, code_idx));
        }
    }
    map
}

impl<'a> RelationInternalSolver<'a> {
    fn get_code_info(&self, code_index: &CodeIndex) -> &CodeInfo {
        match code_index {
            CodeIndex::Src(lang_idx, code_idx) => &self.src_code_infos[*lang_idx][*code_idx],
            CodeIndex::Test(lang_idx, code_idx) => &self.test_code_infos[*lang_idx][*code_idx],
        }
    }

    fn solve_internal_tested_by_dfs(
        &self,
        code_index: &CodeIndex,
        visited: &mut BTreeSet<CodeIndex>,
    ) {
        if visited.contains(code_index) {
            return;
        }
        visited.insert(*code_index);

        let code_info = self.get_code_info(code_index);
        for path_string in &code_info.filepath_dependencies {
            if let Some(next_code_index) = self.path_to_code_index.get(path_string) {
                self.solve_internal_tested_by_dfs(next_code_index, visited);
            } else {
                // TODO: print error if in debug mode
            }
        }
    }

    fn solve_internal(&mut self) -> Result<Relations, String> {
        let source_relations = self
            .test_code_infos
            .iter()
            .enumerate()
            .map(|(lang_idx, tcis)| {
                let src_code_len = self.src_code_infos[lang_idx].len();
                let mut source_relations = vec![
                    SourceRelation {
                        tested_by: Vec::new(),
                    };
                    src_code_len
                ];
                for (i, _tci) in tcis.iter().enumerate() {
                    let code_index = CodeIndex::Test(lang_idx, i);
                    let mut visited = BTreeSet::new();
                    self.solve_internal_tested_by_dfs(&code_index, &mut visited);

                    for ci in visited.iter() {
                        // collect src code depended by the test code.
                        // src code depended by test code is 
                        match ci {
                            CodeIndex::Src(lang_idx2, code_idx) if lang_idx == *lang_idx2 => {
                                source_relations[*code_idx].tested_by.push(i);
                            }
                            _ => {}
                        }
                    }
                }
                source_relations
            })
            .collect();

        Ok(Relations { source_relations })
    }

    fn solve(
        collection: &Collection,
        src_code_infos: &Vec<Vec<CodeInfo>>,
        test_code_infos: &Vec<Vec<CodeInfo>>,
    ) -> Result<Relations, String> {
        // validation
        let lang_len = collection.langs.len();
        if lang_len != src_code_infos.len() || lang_len != test_code_infos.len() {
            return Err(
                "Length of langs, src_code_infos, and test_code_infos are not equal".to_string(),
            );
        }
        for i in 0..lang_len {
            if src_code_infos[i].len() != collection.src_paths[i].len()
                || test_code_infos[i].len() != collection.test_paths[i].len()
            {
                return Err("Length of src_code_infos and test_code_infos are not equal to the length of paths".to_string());
            }
        }

        RelationInternalSolver {
            collection,
            src_code_infos,
            test_code_infos,
            path_to_code_index: create_map_path_to_code_index(
                collection,
                src_code_infos,
                test_code_infos,
            ),
        }
        .solve_internal()
    }
}

// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct SourceRelation {
    pub tested_by: Vec<usize>, // test_code_idx
}

pub struct Relations {
    // source_relations[lang_idx][code_idx]
    pub source_relations: Vec<Vec<SourceRelation>>,
}

// TODO: Re-consider interface: especially src_code_infos and test_code_infos
pub fn solve_relation(
    collection: &Collection,
    src_code_infos: &Vec<Vec<CodeInfo>>,
    test_code_infos: &Vec<Vec<CodeInfo>>,
) -> Result<Relations, String> {
    RelationInternalSolver::solve(collection, src_code_infos, test_code_infos)
}
