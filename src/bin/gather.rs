use std::fs::File;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory path of the repository
    #[arg(long)]
    basepath: String,
    /// Pretty print JSON
    #[arg(long, default_value_t = false)]
    pretty: bool,
}

fn main() {
    let args = Args::parse();

    let base_path = args.basepath;

    if !std::path::Path::new(&base_path).is_dir() {
        eprintln!("{} is not a directory", base_path);
        std::process::exit(1);
    }

    let collection = codelib2_tools::gather_collection(&base_path).unwrap();

    let mut articles = Vec::new();
    for (src_paths, lang) in collection.src_paths.iter().zip(collection.langs.iter()) {
        for src_path in src_paths {
            let total_path = collection.complete_path_str(src_path);
            let file = File::open(&total_path).expect("Failed to open file");
            let commits = vec![];  // TODO:
            let tested_by = vec![];  // TODO:
            let article = codelib2_tools::parse_document_from_file(file, src_path.clone(), lang.clone(), commits, tested_by);
            articles.push(article.unwrap());
        }
    }
    let json_str = if args.pretty {
        serde_json::to_string_pretty(&articles).unwrap()
    } else {
        serde_json::to_string(&articles).unwrap()
    };
    print!("{}", json_str);
}
