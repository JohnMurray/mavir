use derive_builder::Builder;
use tree_sitter::{Node, Parser, Query, QueryCursor, TreeCursor};

use std::fs;
use log::debug;

#[derive(Debug)]
pub struct ParseResult {
    package_name: String,
    import_statements: Vec<String>,
    class_declarations: Vec<ClassDeclarationState>,
}

#[derive(Debug)]
pub enum ParseError {
    ParserInitializationError,
    CannotReadFile(String),
    FileNotParsableAsJava,
    FileProcessingError(String)
}

pub type Result<T> = std::result::Result<T, ParseError>;

pub fn parse_file(file_path: &str) -> Result<ParseResult> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_java::language())
        .map_err(|_| ParseError::ParserInitializationError)?;

    println!("Parsing file: {}", file_path);

    let source_code = fs::read_to_string(file_path)
        .map_err(|_| ParseError::CannotReadFile(file_path.to_string()))?;
    let mut tree = parser.parse(&source_code, None).ok_or(ParseError::FileNotParsableAsJava)?;

    println!("File parsed successfully");
    let root_node = tree.root_node();
    println!("{}", root_node.to_sexp());
    println!("---------");

    let package_name = collect_package(&tree, &source_code)?;
    println!("Package name: {}", package_name);
    let import_statements = collect_import_statements(&tree, &source_code);
    println!("---------");
    let class_declarations = collect_classes(&tree, &source_code)?;
    println!("---------");

    Ok(ParseResult {
        package_name,
        import_statements,
        class_declarations,
    })

}


/// Runs a simple query on the tree to find the package declaration and return the
/// package name. Returns an error if there are issues parsing, but this is not
/// expected to error when processing a valid Java file.
fn collect_package(tree: &tree_sitter::Tree, source_code: &str) -> Result<String> {
    // Query to find package declaration
    let query = Query::new(&tree_sitter_java::language(), r#"
      (package_declaration) @package
    "#).unwrap();
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());
    for m in matches {
        for capture in m.captures {
            let node = capture.node;
            let package_text = &source_code[node.start_byte()..node.end_byte()];

            // Extract the package text by removing the prefix "package " and postfix ";"
            let package_name = package_text
                .strip_prefix("package ")
                .and_then(|p| p.strip_suffix(";"))
                .ok_or(ParseError::FileProcessingError("Malformed package declaration".to_string()))?
                .to_string();
            return Ok(package_name);
        }
    }
    Err(ParseError::FileProcessingError("Could not find package declaration".to_string()))
}

/// Runs a simple query to collect the full text of all the import statements in the
/// Java file. Returns this an a vector of strings with each entry being a single import.
fn collect_import_statements(tree: &tree_sitter::Tree, source_code: &str) -> Vec<String> {
    let query = Query::new(&tree_sitter_java::language(), r#"
      (import_declaration) @import
    "#).unwrap();
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());
    let mut import_statements = Vec::new();
    for m in matches {
        for capture in m.captures {
            let node = capture.node;
            let import_text = &source_code[node.start_byte()..node.end_byte()];
            import_statements.push(import_text.to_string());
        }
    }
    import_statements
}

#[derive(Debug, Builder, Default)]
struct ClassDeclarationState {
    name: String,
    methods: Vec<MethodDeclarationState>,
    parent_chain: Vec<String>,
}

#[derive(Debug, Builder, Default, Clone)]
struct MethodDeclarationState {
    name: String,
    return_type: String,
}

fn collect_classes(tree: &tree_sitter::Tree, source_code: &str) -> Result<Vec<ClassDeclarationState>> {
    // Query to find classes
    let query = Query::new(&tree_sitter_java::language(), r#"
      (class_declaration
        name: (identifier) @class-name
        body: (class_body
          (
            (method_declaration
              (modifiers "abstract")
            ) @method-declaration
          )+
        )
      )
    "#).unwrap();
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

    let mut class_states: Vec<ClassDeclarationState> = vec![];

    'query_match:
    for m in matches {
        let mut state = ClassDeclarationStateBuilder::default();
        let mut methods: Vec<MethodDeclarationState> = vec![];
        for capture in m.captures {
            match query.capture_names()[capture.index as usize] {
                "method-declaration" => {
                    let node = capture.node;
                    let method_name = &source_code[node.start_byte()..node.end_byte()];
                    debug!("Processing method-declaration '{}'", method_name);

                    methods.push(collect_abstract_method(node, source_code)?);
                }
                "class-name" => {
                    let node = capture.node;
                    let class_name = &source_code[node.start_byte()..node.end_byte()];
                    state.name(class_name.to_string());
                    debug!("Processing class-name '{}'", class_name);

                    // Grab the parent node (the class_declaration) and check if this is an
                    // AutoValue (annotated) class. If it's not, then bail out of additional
                    // processing.
                    let parent_node = node.parent().unwrap();
                    let av_class = has_autovalue_annotation(parent_node, source_code, class_name);
                    if !av_class {
                        // If this isn't an AutoValue class, no need to continue processing, move
                        // on to the next match.
                        continue 'query_match;
                    }

                    // Find the class's parent class(es) (if any)
                    state.parent_chain(collect_parent_chain(parent_node, source_code));
                }
                _ => {}
            }
        }

        let state = state
            .methods(methods)
            .build()
            .map_err(|e| ParseError::FileProcessingError(e.to_string()))?;
        debug!("Collected class state: {:#?}", state);
        class_states.push(state);
    }

    Ok(class_states)
}

/// Determines if a given class (specified by the node and class-name) is annotated with
/// an '@AutoValue' annotation.
fn has_autovalue_annotation(node: Node, source_code: &str, class_name: &str) -> bool {
    // Construct a query, using predicates to select match the exact class and annotation name
    let query = Query::new(&tree_sitter_java::language(), &format!(r#"
      (class_declaration
        (modifiers [
            ((marker_annotation) @marker_annotation
              (#eq? @marker_annotation "@AutoValue"))
          ]+
        )
        name: ((identifier) @class-name (#eq? @class-name "{}"))
      )
    "#, class_name)).unwrap();
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, node, source_code.as_bytes());

    // If we have a match, then we found the AutoValue annotation on the desired class. No
    // need to process the match results.
    matches.count() > 0
}

/// Builds up a MethodDeclarationState from the given (method_declaration) node.
fn collect_abstract_method(node: Node, source_code: &str) -> Result<MethodDeclarationState> {
    let query = Query::new(&tree_sitter_java::language(), r#"
      (method_declaration
        type: _ @return-type
        name: (identifier) @method-name
        parameters: (formal_parameters)
      )
    "#).unwrap();
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, node, source_code.as_bytes());

    let mut state = MethodDeclarationStateBuilder::default();
    for m in matches {
        for capture in m.captures {
            let node = capture.node;
            let text = &source_code[node.start_byte()..node.end_byte()];
            match query.capture_names()[capture.index as usize] {
                "return-type" => {
                    state.return_type(text.to_string());
                }
                "method-name" => {
                    state.name(text.to_string());
                }
                _ => {}
            }
        }
    }

    state
        .build()
        .map_err(|e| ParseError::FileProcessingError(e.to_string()))
}

/// Given a node to a class, return the chain of parent classes
fn collect_parent_chain(node: Node, source_code: &str) -> Vec<String> {
    let mut chain: Vec<String> = vec![];
    if node.parent().is_none() {
        return chain;
    }

    let mut current_node = node;
    while current_node.parent().is_some() {
        current_node = current_node.parent().unwrap();
        if current_node.kind() != "class_declaration" {
            continue;
        }

        // current node is a class_declaration, get the class-name
        let name = current_node
            .child_by_field_name("name")
            .map(|n| source_code[n.start_byte()..n.end_byte()].to_string());
        if name.is_some() {
            chain.push(name.unwrap());
        }
    }

    // Reverse the chain so that the first element is the root class
    chain.reverse();
    chain
}