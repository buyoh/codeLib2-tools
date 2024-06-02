use glob::glob;

// TODO: hidden?
#[derive(Debug)]
pub struct Collection {
    base_path: String,
    pub src_paths: Vec<Vec<String>>,
    pub test_paths: Vec<Vec<String>>,
    pub langs: Vec<String>,
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

fn collect_langs(base_path: &str) -> Result<Vec<String>, String> {
    let mut langs = Vec::new();
    let g = if let Ok(g) = glob(&format!("{}/src/*", base_path)) {
        g
    } else {
        return Err("Failed to read glob pattern; base_path may be wrong".to_string());
    };
    for entry in g {
        // if entry is directory
        if let Ok(entry) = entry {
            if entry.is_dir() {
                let lang = entry.file_name().unwrap().to_str().unwrap().to_string();
                langs.push(lang);
            }
        }
    }
    Ok(langs)
}

// TODO: Remove panic
fn collect_paths(
    base_path: &str,
    langs: &Vec<String>,
    src_or_test: &str,
) -> Result<Vec<Vec<String>>, String> {
    let mut src_lang_paths: Vec<Vec<String>> = Vec::new();
    for lang in langs {
        let mut src_paths = Vec::new();
        let g = if let Ok(g) = glob(&format!("{}/{}/{}/**/*", base_path, src_or_test, lang)) {
            g
        } else {
            return Err("Failed to read glob pattern; base_path may be wrong".to_string());
        };
        for entry in g {
            if let Ok(entry) = entry {
                // if entry is file
                if entry.is_file() {
                    let stripped_path = entry.strip_prefix(base_path).unwrap();
                    src_paths.push(format!("/{}", stripped_path.to_str().unwrap()));  // add '/' to the beginning
                }
            }
        }
        src_lang_paths.push(src_paths);
    }
    Ok(src_lang_paths)
}

pub fn gather_collection(base_path: &str) -> Result<Collection, String> {
    let langs = match collect_langs(base_path) {
        Ok(langs) => langs,
        Err(err) => return Err(err),
    };
    let src_paths = match collect_paths(base_path, &langs, "src") {
        Ok(src_paths) => src_paths,
        Err(err) => return Err(err),
    };
    let test_paths = match collect_paths(base_path, &langs, "test") {
        Ok(test_paths) => test_paths,
        Err(err) => return Err(err),
    };

    let collection = Collection {
        base_path: base_path.to_string(),
        langs: langs.clone(),
        src_paths,
        test_paths,
    };
    Ok(collection)
}

pub fn gather_commit_info(_filepath: &str) -> Result<Vec<crate::codelib::Commit>, String> {
    // TODO: implement
    Ok(Vec::new())
}
