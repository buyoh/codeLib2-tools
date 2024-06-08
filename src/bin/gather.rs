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

    let articles = match codelib2_tools::complete_articles(&base_path) {
        Ok(articles) => articles,
        Err(err) => {
            eprintln!("Failed: {}", err);
            std::process::exit(1);
        }
    };

    let json_str = if args.pretty {
        serde_json::to_string_pretty(&articles).unwrap()
    } else {
        serde_json::to_string(&articles).unwrap()
    };
    print!("{}", json_str);
}
