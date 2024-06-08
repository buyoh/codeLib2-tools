#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Commit {
    pub sha: String,
    pub date: String,
    pub message: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Article {
    pub title: String,
    pub overview: String,
    pub code: String,
    pub lang: String,
    pub path: String,
    pub require: String,
    pub references: Vec<String>,
    pub words: Vec<String>,
    pub verified: Vec<String>,
    pub commits: Vec<Commit>,
    pub tested_by: Vec<String>,
}

// ------------------------------------

#[derive(Debug)]
pub struct CodeInfo {
    pub filepath_dependencies: Vec<String>,
}

pub struct CodeInfoSets {
    pub lang: String,
    pub src_code_infos: Vec<CodeInfo>,
    pub test_code_infos: Vec<CodeInfo>,
}

// ------------------------------------

#[derive(Debug)]
pub struct SourceSets {
    // TODO: Rename to SourcePathSets
    pub lang: String,
    pub src_paths: Vec<String>,
    pub test_paths: Vec<String>,
}

pub struct Collection {
    pub base_path: String,
    pub source_sets: Vec<SourceSets>,
}

impl Collection {
    pub fn complete_path_str(&self, path: &str) -> String {
        if path.starts_with("/") {
            format!("{}{}", self.base_path, path)
        } else {
            format!("{}/{}", self.base_path, path)
        }
    }
}
