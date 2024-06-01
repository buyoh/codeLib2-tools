use std::fs::File;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // #[arg(short, long)]
    // name: String,

    #[arg(short, long, default_value_t = 1)]
    count: u8,

    #[arg(long)]  // TODO: for testing. remove this.
    filepath: String,
}

fn main() {
    let args = Args::parse();

    // let path = "target/codelib_full.json";

    // let codelibs = codelib2_tools::from_json_array(&std::fs::read_to_string(path).unwrap());

    // for _ in 0..args.count {
    //     println!("Hello {}!", args.name)
    // }

    // for codelib in codelibs {
    //     println!("{:?}", codelib);
    // }
    let article_path = args.filepath;
    let file = File::open(article_path).expect("Failed to open file");
    let article = codelib2_tools::parse_document_from_file(file, "rust".to_string(), article_path);

    // print article
    println!("{:?}", article);
}
