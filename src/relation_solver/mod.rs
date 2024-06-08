use std::collections::{BTreeMap, BTreeSet};

use crate::{CodeInfo, CodeInfoSets, Collection};

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
    code_info_sets_vec: &'a Vec<CodeInfoSets>,

    // source_relations: Vec<SourceRelationSolving>,
    path_to_code_index: BTreeMap<String, CodeIndex>,
}

fn create_map_path_to_code_index(collection: &Collection) -> BTreeMap<String, CodeIndex> {
    let mut map = BTreeMap::new();
    for (lang_idx, source_sets) in collection.source_sets.iter().enumerate() {
        for (code_idx, src_path) in source_sets.src_paths.iter().enumerate() {
            map.insert(src_path.clone(), CodeIndex::Src(lang_idx, code_idx));
        }
        for (code_idx, test_path) in source_sets.test_paths.iter().enumerate() {
            map.insert(test_path.clone(), CodeIndex::Test(lang_idx, code_idx));
        }
    }
    map
}

impl<'a> RelationInternalSolver<'a> {
    fn get_code_info(&self, code_index: &CodeIndex) -> &CodeInfo {
        match code_index {
            CodeIndex::Src(lang_idx, code_idx) => {
                &self.code_info_sets_vec[*lang_idx].src_code_infos[*code_idx]
            }
            CodeIndex::Test(lang_idx, code_idx) => {
                &self.code_info_sets_vec[*lang_idx].test_code_infos[*code_idx]
            }
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
            .collection
            .source_sets
            .iter()
            .enumerate()
            .map(|(lang_idx, source_sets)| {
                let code_info_sets = &self.code_info_sets_vec[lang_idx];
                let src_code_len = source_sets.src_paths.len();
                let mut source_relations = vec![
                    SourceRelation {
                        tested_by: Vec::new(),
                    };
                    src_code_len
                ];
                for (i, _tci) in code_info_sets.test_code_infos.iter().enumerate() {
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
        code_info_sets_vec: &Vec<CodeInfoSets>,
    ) -> Result<Relations, String> {
        // validation
        // TODO: Modfy struct to be unecessary this validation

        if collection.source_sets.len() != code_info_sets_vec.len() {
            return Err("Length of source_sets and code_info_sets are not equal".to_string());
        }

        for (source_sets, code_info_sets) in collection.source_sets.iter().zip(code_info_sets_vec) {
            if source_sets.lang != code_info_sets.lang {
                return Err(
                    "Langs in source_sets and code_info_sets are not equal. Is it sorted?"
                        .to_string(),
                );
            }
        }

        RelationInternalSolver {
            collection,
            code_info_sets_vec,
            path_to_code_index: create_map_path_to_code_index(collection),
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
    code_info_sets_vec: &Vec<CodeInfoSets>,
) -> Result<Relations, String> {
    RelationInternalSolver::solve(collection, code_info_sets_vec)
}
