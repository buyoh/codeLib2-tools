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
    /// Output article json file
    #[arg(long)]
    output_path_article: Option<String>,
    /// Output collection json file
    #[arg(long)]
    output_path_collection: Option<String>,
}

fn main() {
    let args = Args::parse();

    let base_path = args.basepath;

    let writer_article = if let Some(output_path_article) = args.output_path_article {
        let writer = std::fs::File::create(output_path_article).unwrap();
        Some(writer)
    } else {
        None
    };

    let writer_collection = if let Some(output_path_collection) = args.output_path_collection {
        let writer = std::fs::File::create(output_path_collection).unwrap();
        Some(writer)
    } else {
        None
    };

    if !std::path::Path::new(&base_path).is_dir() {
        eprintln!("{} is not a directory", base_path);
        std::process::exit(1);
    }

    let collection = match codelib2_tools::gather_collection(&base_path) {
        Ok(collection) => collection,
        Err(err) => {
            eprintln!("Failed: {}", err);
            std::process::exit(1);
        }
    };

    if let Some(writer_collection) = writer_collection {
        if args.pretty {
            serde_json::to_writer_pretty(writer_collection, &collection).unwrap();
        } else {
            serde_json::to_writer(writer_collection, &collection).unwrap();
        };
    }

    let articles = match codelib2_tools::complete_articles(&collection) {
        Ok(articles) => articles,
        Err(err) => {
            eprintln!("Failed: {}", err);
            std::process::exit(1);
        }
    };

    if let Some(writer_article) = writer_article {
        if args.pretty {
            serde_json::to_writer_pretty(writer_article, &articles).unwrap();
        } else {
            serde_json::to_writer(writer_article, &articles).unwrap();
        };
    }
}
