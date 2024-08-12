mod parse;
mod generate;
mod util;

use anyhow::{anyhow, Result};
use clap::Parser;
use env_logger;
use log::{info, LevelFilter};
use crate::parse::ParseResult;

/// Generates AutoValue classes for given Java files. Outputs generated code as a source JAR.
#[derive(clap::Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to a Java source file.
    #[arg(short, long)]
    file_path: Vec<String>,

    /// Path to the output file that will contain the generated code. This should be
    /// a path to a source JAR. The path MUST not exist, but the parent directory is
    /// expected to exist.
    #[arg(short, long)]
    output_path: String,

    /// Print Verbose output. This can also be configured with 'RUST_LOG=debug'
    #[arg(short, long)]
    verbose: bool,
}


fn main() -> Result<()> {
    // Parse the CLI arguments and configure the log-level
    let args = Args::parse();
    let mut builder = env_logger::builder();
    if args.verbose {
        builder.filter_level(LevelFilter::Debug);
        println!("{:?}", args);
    }
    builder.init();

    if args.file_path.is_empty() {
        return Err(anyhow!("Must specify at least one --file-path option"));
    }

    let parse_results = args.file_path
        .iter()
        .map(|file_path| {
            info!("Generating code for: {}", file_path);
            parse::parse_file(file_path)
        })
        .collect::<parse::Result<Vec<ParseResult>>>()?;
    generate::generate_code(parse_results, &args.output_path)?;

    Ok(())
}
