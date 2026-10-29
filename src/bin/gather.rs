use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use serde::Serialize;

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
    /// Exit with failure (without writing outputs) if any document is invalid
    #[arg(long, default_value_t = false)]
    strict: bool,
}

// Outputs written into temporary files. They are renamed to the destination by commit(),
// otherwise removed on drop, so that outputs are never left broken or inconsistent with each other.
struct PendingOutputs {
    // (temporary path, destination path)
    files: Vec<(String, String)>,
    pretty: bool,
}

impl PendingOutputs {
    fn new(pretty: bool) -> Self {
        Self {
            files: Vec::new(),
            pretty,
        }
    }

    fn write<T: Serialize>(&mut self, path: &str, value: &T) -> Result<(), String> {
        if self.files.iter().any(|(_, p)| p == path) {
            return Err(format!("{}: specified as multiple outputs", path));
        }
        let tmp_path = format!("{}.{}-{}.tmp", path, std::process::id(), self.files.len());
        // Register first to remove it on failure
        self.files.push((tmp_path.clone(), path.to_string()));
        let file = File::create(&tmp_path).map_err(|e| format!("{}: {}", tmp_path, e))?;
        let mut writer = BufWriter::new(file);
        if self.pretty {
            serde_json::to_writer_pretty(&mut writer, value)
        } else {
            serde_json::to_writer(&mut writer, value)
        }
        .map_err(|e| format!("{}: {}", tmp_path, e))?;
        writer.flush().map_err(|e| format!("{}: {}", tmp_path, e))
    }

    fn commit(mut self) -> Result<(), String> {
        let mut renamed = Vec::new();
        while !self.files.is_empty() {
            let (tmp_path, path) = &self.files[0];
            // On failure, remaining temporary files are removed on drop
            std::fs::rename(tmp_path, path)
                .map_err(|e| format!("{}: {} (already updated: {:?})", path, e, renamed))?;
            renamed.push(self.files.remove(0).1);
        }
        Ok(())
    }
}

impl Drop for PendingOutputs {
    fn drop(&mut self) {
        for (tmp_path, _) in &self.files {
            let _ = std::fs::remove_file(tmp_path);
        }
    }
}

fn run(args: Args) -> Result<ExitCode, String> {
    let base_path = args.basepath;
    if !Path::new(&base_path).is_dir() {
        return Err(format!("{} is not a directory", base_path));
    }

    let (collection, warnings) = codelib2_tools::gather_collection(&base_path)?;
    for warning in &warnings {
        eprintln!("Warning: {}", warning);
    }

    let result = codelib2_tools::complete_articles(&collection)?;
    for path in &result.skipped {
        eprintln!("Skipped (no document): {}", path);
    }
    for (path, warning) in &result.warnings {
        eprintln!("Warning: {}: {}", path, warning);
    }
    for (path, err) in &result.errors {
        eprintln!("Failed to parse article: {}: {}", path, err);
    }

    if args.strict && !result.errors.is_empty() {
        eprintln!("Failed: {} invalid document(s)", result.errors.len());
        return Ok(ExitCode::FAILURE);
    }

    let mut outputs = PendingOutputs::new(args.pretty);
    if let Some(path) = &args.output_path_collection {
        outputs.write(path, &collection)?;
    }
    if let Some(path) = &args.output_path_article {
        outputs.write(path, &result.articles)?;
    }
    outputs.commit()?;
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Failed: {}", err);
            ExitCode::FAILURE
        }
    }
}
