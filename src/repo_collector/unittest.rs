use super::gather_collection;
use crate::Lang;
use std::fs;
use std::path::{Path, PathBuf};

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        // Contains glob metacharacters and spaces
        let path = std::env::temp_dir().join(format!(
            "codelib2-tools-test [{}] {}",
            std::process::id(),
            name
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn write(&self, relative_path: &str, content: &str) {
        let path = self.0.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn test_gather_collection() {
    let dir = TempDir::new("gather_collection");
    dir.write("src/cpp/b/y.hpp", "");
    dir.write("src/cpp/a.hpp", "");
    dir.write("src/cpp/.clang-format", "");
    dir.write("src/Go/x.go", "");
    dir.write("src/ruby/x.rb", "");
    dir.write("src/ruby/.rubocop.yml", "");
    dir.write("src/python/x.py", "");
    dir.write("src/template.hpp", "");
    dir.write("test/cpp/t.cpp", "");
    dir.write("test/cpp/input.txt", "");

    let base_path = dir.path().to_str().unwrap();
    let (collection, warnings) = gather_collection(base_path).unwrap();

    assert_eq!(collection.base_path, base_path);
    let summary: Vec<_> = collection
        .source_sets
        .iter()
        .map(|s| (s.lang, s.src_paths.clone(), s.test_paths.clone()))
        .collect();
    assert_eq!(
        summary,
        vec![
            (
                Lang::Cpp,
                vec!["/src/cpp/a.hpp".to_string(), "/src/cpp/b/y.hpp".to_string()],
                vec!["/test/cpp/t.cpp".to_string()]
            ),
            (Lang::Ruby, vec!["/src/ruby/x.rb".to_string()], vec![]),
            (Lang::Go, vec!["/src/Go/x.go".to_string()], vec![]),
        ]
    );
    assert_eq!(warnings.len(), 1, "{:?}", warnings);
    assert!(warnings[0].contains("python"));
}

#[test]
fn test_gather_collection_without_src() {
    let dir = TempDir::new("without_src");
    let err = gather_collection(dir.path().to_str().unwrap()).unwrap_err();
    assert!(err.contains("src"), "{}", err);
}

#[cfg(unix)]
#[test]
fn test_gather_collection_symlink_loop() {
    let dir = TempDir::new("symlink_loop");
    dir.write("src/cpp/a.hpp", "");
    std::os::unix::fs::symlink(dir.path().join("src/cpp"), dir.path().join("src/cpp/loop"))
        .unwrap();
    let err = gather_collection(dir.path().to_str().unwrap()).unwrap_err();
    assert!(err.contains("too deep"), "{}", err);
}
