#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Cpp,
    Rust,
    Ruby,
    Javascript,
    Typescript,
    Go,
}

impl Lang {
    pub const ALL: [Lang; 6] = [
        Lang::Cpp,
        Lang::Rust,
        Lang::Ruby,
        Lang::Javascript,
        Lang::Typescript,
        Lang::Go,
    ];

    // Directory name under src/ (case-insensitive, e.g. "Go")
    pub fn from_dir_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|lang| lang.name().eq_ignore_ascii_case(name))
    }

    pub fn name(&self) -> &'static str {
        match self {
            Lang::Cpp => "cpp",
            Lang::Rust => "rust",
            Lang::Ruby => "ruby",
            Lang::Javascript => "javascript",
            Lang::Typescript => "typescript",
            Lang::Go => "go",
        }
    }

    // File extensions collected as source code
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Lang::Cpp => &["hpp", "cpp", "h", "cc"],
            Lang::Rust => &["rs"],
            Lang::Ruby => &["rb", "ruby"],
            Lang::Javascript => &["js", "mjs", "cjs"],
            Lang::Typescript => &["ts"],
            Lang::Go => &["go"],
        }
    }
}

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
    pub lang: Lang,
    pub path: String,
    pub require: String,
    pub references: Vec<String>,
    pub words: Vec<String>,
    pub verified: Vec<String>,
    pub commits: Vec<Commit>,
    pub tested_by: Vec<String>,
}

// ------------------------------------

#[derive(Debug, Default)]
pub struct CodeInfo {
    pub filepath_dependencies: Vec<String>,
}

// ------------------------------------

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SourceSets {
    // TODO: Rename to SourcePathSets
    pub lang: Lang,
    pub src_paths: Vec<String>,
    pub test_paths: Vec<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
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
