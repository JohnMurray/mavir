#[macro_use]
extern crate derive_builder;

mod parse;
mod generate;
mod util;

use clap::Parser;
use env_logger;
use log::LevelFilter;

/// Generates AutoValue classes for given Java files. Outputs generated code as a source JAR.
#[derive(clap::Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to a Java source file.
    #[arg(short, long)]
    file_paths: Vec<String>,

    /// Path to the output file that will contain the generated code. This should be
    /// a path to a source JAR. The path MUST not exist, but the parent directory is
    /// expected to exist.
    #[arg(short, long)]
    output_path: String,

    /// Print Verbose output. This can also be configured with 'RUST_LOG=debug'
    #[arg(short, long)]
    verbose: bool,
}


fn main() {
    // Parse the CLI arguments and configure the log-level
    let args = Args::parse();
    let mut builder = env_logger::builder();
    if args.verbose {
        builder.filter_level(LevelFilter::Debug);
        println!("{:?}", args);
    }
    builder.init();

    let parse_result = parse::parse_file(args.file_paths[0].as_str()).unwrap();
    generate::generate_code(parse_result, args.output_path.as_str()).unwrap();
}
