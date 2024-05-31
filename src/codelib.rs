
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Commit {
  sha: String,
  date: String,
  message: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Article {
  pub title: String,
  pub overview: String,
  pub code: String,
  pub lang: String,
  pub path: String,
  pub require: Option<String>,
  pub references: Vec<String>,
  pub words: Vec<String>,
  pub verified: Vec<String>,
  pub commits: Vec<Commit>,
  pub tested_by: Vec<String>,
}