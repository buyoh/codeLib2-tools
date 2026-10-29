use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const GATHER: &str = env!("CARGO_BIN_EXE_gather");

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "codelib2-tools-gather [{}] {}",
            std::process::id(),
            name
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn write(&self, relative_path: &str, content: &[u8]) {
        let path = self.0.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const DOC: &str = "\
// %=BEGIN DOC
// %title
// A
// %words
// x, y
// %=END DOC
// %=BEGIN CODE
int a();
// %=END CODE
";

fn run_gather(repo: &TempDir, out: &TempDir, extra_args: &[&str]) -> Output {
    Command::new(GATHER)
        .arg("--basepath")
        .arg(repo.path())
        .arg("--output-path-article")
        .arg(out.file("article.json"))
        .arg("--output-path-collection")
        .arg(out.file("collection.json"))
        .args(extra_args)
        .output()
        .unwrap()
}

fn read_json(path: PathBuf) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn test_gather() {
    let repo = TempDir::new("repo");
    let out = TempDir::new("out");
    repo.write("src/cpp/a.hpp", DOC.as_bytes());
    repo.write("src/cpp/empty.hpp", b"");
    repo.write("test/cpp/a.cpp", b"#include \"src/cpp/a.hpp\"\n");

    let output = run_gather(&repo, &out, &["--strict"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{}", stderr);
    assert!(
        stderr.contains("Skipped (no document): /src/cpp/empty.hpp"),
        "{}",
        stderr
    );

    let articles = read_json(out.file("article.json"));
    assert_eq!(articles.as_array().unwrap().len(), 1);
    assert_eq!(articles[0]["path"], "/src/cpp/a.hpp");
    assert_eq!(articles[0]["lang"], "cpp");
    assert_eq!(articles[0]["words"], serde_json::json!(["x", "y"]));
    assert_eq!(
        articles[0]["tested_by"],
        serde_json::json!(["/test/cpp/a.cpp"])
    );

    let collection = read_json(out.file("collection.json"));
    assert_eq!(collection["source_sets"][0]["lang"], "cpp");
    assert_eq!(
        fs::read_dir(out.path()).unwrap().count(),
        2,
        "no temporary file is left"
    );
}

#[test]
fn test_invalid_document() {
    let repo = TempDir::new("invalid");
    let out = TempDir::new("invalid_out");
    repo.write("src/cpp/a.hpp", DOC.as_bytes());
    repo.write("src/cpp/b.hpp", b"// %=BEGIN CODE\nint b();\n");

    // Not strict: invalid documents are reported and excluded
    let output = run_gather(&repo, &out, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{}", stderr);
    assert!(stderr.contains("/src/cpp/b.hpp"), "{}", stderr);
    assert_eq!(
        read_json(out.file("article.json"))
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // Strict: fail without touching existing outputs
    fs::write(out.file("article.json"), "previous").unwrap();
    let output = run_gather(&repo, &out, &["--strict"]);
    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(out.file("article.json")).unwrap(),
        "previous"
    );
}

#[test]
fn test_invalid_utf8_is_error() {
    let repo = TempDir::new("utf8");
    let out = TempDir::new("utf8_out");
    repo.write(
        "src/cpp/a.hpp",
        b"// %=BEGIN CODE\n\xff\xfe\n// %=END CODE\n",
    );

    let output = run_gather(&repo, &out, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("a.hpp"), "{}", stderr);
    assert!(!out.file("article.json").exists());
}

#[test]
fn test_basepath_not_found_keeps_outputs() {
    let out = TempDir::new("notfound_out");
    fs::write(out.file("article.json"), "previous").unwrap();

    let output = Command::new(GATHER)
        .arg("--basepath")
        .arg(out.file("not_found"))
        .arg("--output-path-article")
        .arg(out.file("article.json"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(out.file("article.json")).unwrap(),
        "previous"
    );
}

#[test]
fn test_output_dir_not_found() {
    let repo = TempDir::new("outdir");
    repo.write("src/cpp/a.hpp", DOC.as_bytes());
    let output = Command::new(GATHER)
        .arg("--basepath")
        .arg(repo.path())
        .arg("--output-path-article")
        .arg(repo.file("not_found/article.json"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("not_found"), "{}", stderr);
}

#[test]
fn test_outputs_are_updated_together() {
    let repo = TempDir::new("together");
    let out = TempDir::new("together_out");
    repo.write("src/cpp/a.hpp", DOC.as_bytes());
    fs::write(out.file("collection.json"), "previous").unwrap();

    // Writing article fails, so collection must not be updated either
    let output = Command::new(GATHER)
        .arg("--basepath")
        .arg(repo.path())
        .arg("--output-path-collection")
        .arg(out.file("collection.json"))
        .arg("--output-path-article")
        .arg(out.file("not_found/article.json"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(out.file("collection.json")).unwrap(),
        "previous"
    );
    assert_eq!(
        fs::read_dir(out.path()).unwrap().count(),
        1,
        "no temporary file is left"
    );
}

#[test]
fn test_same_output_paths_are_rejected() {
    let repo = TempDir::new("same");
    let out = TempDir::new("same_out");
    repo.write("src/cpp/a.hpp", DOC.as_bytes());

    let output = Command::new(GATHER)
        .arg("--basepath")
        .arg(repo.path())
        .arg("--output-path-collection")
        .arg(out.file("x.json"))
        .arg("--output-path-article")
        .arg(out.file("x.json"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read_dir(out.path()).unwrap().count(), 0);
}

#[test]
fn test_rename_failure_removes_temporary_files() {
    let repo = TempDir::new("rename");
    let out = TempDir::new("rename_out");
    repo.write("src/cpp/a.hpp", DOC.as_bytes());
    // A directory cannot be replaced by a file
    fs::create_dir(out.file("article.json")).unwrap();

    let output = run_gather(&repo, &out, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("already updated"), "{}", stderr);
    let names: Vec<_> = fs::read_dir(out.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert!(names.iter().all(|n| !n.ends_with(".tmp")), "{:?}", names);
}
