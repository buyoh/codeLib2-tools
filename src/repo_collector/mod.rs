use std::fs;
use std::path::Path;

use crate::{Collection, Lang, SourceSets};

#[cfg(test)]
mod unittest;

fn file_name_str(path: &Path) -> Result<&str, String> {
    path.file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("{}: file name is not valid UTF-8", path.display()))
}

// Pairs of language and its directory name (e.g. "Go")
type LangDirs = Vec<(Lang, String)>;

// Directories under src/ are languages. Returns found languages and warnings.
fn collect_langs(base_path: &Path) -> Result<(LangDirs, Vec<String>), String> {
    let src_dir = base_path.join("src");
    let mut entries = fs::read_dir(&src_dir)
        .map_err(|e| format!("{}: {}", src_dir.display(), e))?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{}: {}", src_dir.display(), e))?;
    // Sort to be deterministic when duplicated directories exist (e.g. "Go" and "go")
    entries.sort();

    let mut langs: LangDirs = Vec::new();
    let mut warnings = Vec::new();
    for path in entries {
        if !path.is_dir() {
            continue;
        }
        let name = file_name_str(&path)?;
        match Lang::from_dir_name(name) {
            Some(lang) if langs.iter().any(|(l, _)| *l == lang) => {
                warnings.push(format!("{}: duplicated language directory", path.display()))
            }
            Some(lang) => langs.push((lang, name.to_string())),
            None => warnings.push(format!(
                "{}: unsupported language directory",
                path.display()
            )),
        }
    }
    langs.sort();
    Ok((langs, warnings))
}

// Guard against symlink loops
const MAX_DIR_DEPTH: usize = 32;

// Collect files recursively (following symlinks) as paths relative to base_path, e.g. "/src/cpp/a.hpp".
fn collect_files(
    dir: &Path,
    relative_dir: &str,
    extensions: &[&str],
    depth: usize,
    paths: &mut Vec<String>,
) -> Result<(), String> {
    if depth > MAX_DIR_DEPTH {
        return Err(format!(
            "{}: directory is too deep (symlink loop?)",
            dir.display()
        ));
    }
    let mut entries = fs::read_dir(dir)
        .map_err(|e| format!("{}: {}", dir.display(), e))?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{}: {}", dir.display(), e))?;
    entries.sort();

    for path in entries {
        let relative_path = format!("{}/{}", relative_dir, file_name_str(&path)?);
        if path.is_dir() {
            collect_files(&path, &relative_path, extensions, depth + 1, paths)?;
        } else if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if extensions.contains(&ext) {
                paths.push(relative_path);
            }
        }
    }
    Ok(())
}

fn collect_paths(
    base_path: &Path,
    lang_dir: &str,
    src_or_test: &str,
    lang: Lang,
) -> Result<Vec<String>, String> {
    let dir = base_path.join(src_or_test).join(lang_dir);
    let mut paths = Vec::new();
    if dir.is_dir() {
        collect_files(
            &dir,
            &format!("/{}/{}", src_or_test, lang_dir),
            lang.extensions(),
            0,
            &mut paths,
        )?;
    }
    Ok(paths)
}

// Returns the collection and warnings
pub fn gather_collection(base_path: &str) -> Result<(Collection, Vec<String>), String> {
    let base = Path::new(base_path);
    let (langs, warnings) = collect_langs(base)?;

    let source_sets = langs
        .into_iter()
        .map(|(lang, lang_dir)| {
            Ok(SourceSets {
                lang,
                src_paths: collect_paths(base, &lang_dir, "src", lang)?,
                test_paths: collect_paths(base, &lang_dir, "test", lang)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let collection = Collection {
        base_path: base_path.to_string(),
        source_sets,
    };
    Ok((collection, warnings))
}
