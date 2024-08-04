#[macro_use]
extern crate derive_builder;

mod parse;

use clap::Parser;


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

    /// Print Verbose output
    #[arg(short, long)]
    verbose: bool,
}


fn main() {
    let args = Args::parse();
    if args.verbose {
        println!("{:?}", args);
    }

    parse::parse_file(args.file_paths[0].as_str());

    // let mut parser = Parser::new();
    // parser.set_language(&tree_sitter_java::language()).expect("Error loading Java grammar");
    // let source_code = "class HelloWorld { public static void main(String[] Args) { System.out.println(\"Hello, World!\"); } }";
    // let mut tree = parser.parse(source_code, None).unwrap();
    // let root_node = tree.root_node();
    // println!("{}", root_node.to_sexp());
}
