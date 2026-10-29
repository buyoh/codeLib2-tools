use std::collections::{BTreeMap, BTreeSet};

use crate::CodeInfo;

#[cfg(test)]
mod unittest;

pub struct CodeNode<'a> {
    pub path: &'a str,
    pub code_info: &'a CodeInfo,
}

// Returns a map from src path to test paths which depend on it (directly or indirectly).
// Test paths are in the order of `tests`. Src codes not tested are not contained.
pub fn solve_tested_by(srcs: &[CodeNode], tests: &[CodeNode]) -> BTreeMap<String, Vec<String>> {
    let src_paths: BTreeSet<&str> = srcs.iter().map(|n| n.path).collect();
    let code_infos: BTreeMap<&str, &CodeInfo> = srcs
        .iter()
        .chain(tests.iter())
        .map(|n| (n.path, n.code_info))
        .collect();

    let mut tested_by: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for test in tests {
        // DFS from the test code
        let mut visited = BTreeSet::from([test.path]);
        let mut stack = vec![test.path];
        while let Some(path) = stack.pop() {
            let Some(code_info) = code_infos.get(path) else {
                // TODO: print error if in debug mode
                continue;
            };
            for dep in &code_info.filepath_dependencies {
                if visited.insert(dep.as_str()) {
                    stack.push(dep.as_str());
                }
            }
        }

        for path in visited {
            if path != test.path && src_paths.contains(path) {
                tested_by
                    .entry(path.to_string())
                    .or_default()
                    .push(test.path.to_string());
            }
        }
    }
    tested_by
}
