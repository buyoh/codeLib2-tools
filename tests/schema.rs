// Check that spec/schema.json is consistent with the output of gather
use codelib2_tools::{Article, Commit, Lang};
use std::collections::BTreeSet;

fn load_schema() -> serde_json::Value {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/spec/schema.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn keys(value: &serde_json::Value) -> BTreeSet<String> {
    value.as_object().unwrap().keys().cloned().collect()
}

fn strings(value: &serde_json::Value) -> BTreeSet<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn test_schema_is_consistent_with_article() {
    let schema = load_schema();
    let article = Article {
        title: String::new(),
        overview: String::new(),
        code: String::new(),
        lang: Lang::Cpp,
        path: String::new(),
        require: String::new(),
        references: vec![],
        words: vec![],
        verified: vec![],
        commits: vec![Commit {
            sha: String::new(),
            date: String::new(),
            message: String::new(),
        }],
        tested_by: vec![],
    };
    let article = serde_json::to_value(&article).unwrap();

    assert_eq!(keys(&schema["properties"]), keys(&article));
    assert!(strings(&schema["required"]).is_subset(&keys(&article)));
    assert_eq!(
        keys(&schema["properties"]["commits"]["items"]["properties"]),
        keys(&article["commits"][0])
    );
}

#[test]
fn test_schema_lang_enum() {
    let schema = load_schema();
    let langs: BTreeSet<String> = Lang::ALL
        .iter()
        .map(|lang| {
            serde_json::to_value(lang)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(strings(&schema["properties"]["lang"]["enum"]), langs);
    for lang in Lang::ALL {
        assert_eq!(serde_json::to_value(lang).unwrap(), lang.name());
    }
}
