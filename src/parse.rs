use derive_builder::Builder;
use tree_sitter::{Node, Parser, Query, QueryCursor, TreeCursor};

use std::fs;

pub struct ParseState {}

#[derive(Debug, Builder, Default)]
pub struct AutoValueClass {
    class_name: String,
}

pub enum ParseError {
    ParserInitializationError,
    CannotReadFile(String),
    FileNotParsableAsJava,
    FileProcessingError(String)
}

pub type Result<T> = std::result::Result<T, ParseError>;

pub fn parse_file(file_path: &str) -> Result<ParseState> {
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
    collect_annotations(&tree, &source_code);
    println!("---------");

    // let mut cursor = tree.walk();
    // let mut state = CursorState::default();
    // traverse_tree(&mut cursor, &source_code, 0, &mut state);

    // TODO: Finalize
    Ok(ParseState {})

    // let source_code = "class HelloWorld { public static void main(String[] Args) { System.out.println(\"Hello, World!\"); } }";
    // let mut tree = parser.parse(source_code, None).unwrap();
    // let root_node = tree.root_node();
    // println!("{}", root_node.to_sexp());
}

#[derive(Debug, Builder, Default)]
struct ClassDeclarationState {
    name: String,
    autovalue_class: bool,
}

#[derive(Default)]
struct CursorState {
    classes: Vec<ClassDeclarationStateBuilder>,

    final_classes: Vec<ClassDeclarationState>,
}

/*
// fn traverse_tree(cursor: &mut TreeCursor, source: &str, depth: usize, state: &mut CursorState) -> Result<()> {
//     loop {
//         // print_node(cursor, source, depth);
//
//         // If a node is a class declaration, register a state builder
//         let node = cursor.node();
//
//         if node.kind() == "class_declaration" {
//             traverse_class(cursor, source, depth, state)?;
//             println!("Found class declaration");
//         } else {
//             // If this node has children, traverse them
//             if cursor.goto_first_child() {
//                 traverse_tree(cursor, source, depth + 1, state);
//                 cursor.goto_parent();
//             }
//         }
//
//         // Move to the next sibling
//         if !cursor.goto_next_sibling() {
//             break;
//         }
//
//     }
//     Ok(())
// }
//
// fn traverse_class(cursor: &mut TreeCursor, source: &str, depth: usize, state: &mut CursorState) -> Result<()> {
//     let mut builder = ClassDeclarationStateBuilder::default();
//     builder.autovalue_class(true);
//     builder.name("boop".to_string());
//     state.classes.push(builder);
//
//     if cursor.goto_first_child() {
//         let node = cursor.node();
//         match node.kind() {
//             "modifiers" => {}
//             // declare all the top-level fields in class so we can hand off to specific traverse functions
//             _ => {}
//         }
//         traverse_tree(cursor, source, depth + 1, state)?;
//         cursor.goto_parent();
//     } else {
//         return Err(ParseError::FileProcessingError("Class declaration has no children".to_string()));
//     }
//
//     state.final_classes.push(state.classes.pop().unwrap().build().unwrap());
//     Ok(())
// }
//
// fn print_node(cursor: &TreeCursor, source: &str, depth: usize) {
//     let node = cursor.node();
//     let indent = "  ".repeat(depth);
//     let node_text = &source[node.start_byte()..node.end_byte()];
//     let preview = node_text.lines().next().unwrap_or(node_text);
//     println!("{}{}:{} - {}", indent, node.kind(), node.start_position().row + 1, preview);
// }
*/

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


struct MatchState {

}

/*
 *  Idea:
 *    - I'm going to collect all the classes
 *    - For each class call another method to collect annotations
 *        - When calling the method pass in the node for the class so the query only searches
 *          the class scope
 *    - Filter these classes based on annotations
 *    - For the remainder, call another method to collect method declarations
 *    -
 */

fn collect_annotations(tree: &tree_sitter::Tree, source_code: &str) {
    // Query to find annotations and their targets
    let query = Query::new(&tree_sitter_java::language(), r#"
      (class_declaration
        (modifiers [
            (annotation) @annotation
            (marker_annotation) @marker_annotation
          ]+
        )
        name: (identifier) @class_name
        body: (class_body
          (method_declaration
            (modifiers [
                ["abstract" "public" "private" "protected"] @method_visibility
                (marker_annotation
                    name: (identifier) @method_marker_annotation)
                (annotation
                    name: (identifier) @method_annotation)]* )?
            type: [
                (type_identifier) @method_return_type
                (generic_type) @method_generic_return_type
            ]
            name: (identifier) @method_name
          )*
        )
      )
    "#).unwrap();

    /*
            parameters: (formal_parameters) @method_parameters
     */

    let mut query_cursor = QueryCursor::new();
    let matches = query_cursor.matches(&query, tree.root_node(), source_code.as_bytes());

    for match_ in matches {
        println!("Match {:#?}", match_);
        for capture in match_.captures {
            println!("  Capture ({}) {:#?}",
                     query.capture_names()[capture.index as usize],
                     &source_code[capture.node.start_byte()..capture.node.end_byte()]);
        }
        // let mut annotation = None;
        // let mut annotated_element = None;

        // for capture in match_.captures {
        //     match capture.index {
        //         0 => annotation = Some(capture.node),
        //         1 | 2 | 3 => annotated_element = Some(capture.node),
        //         _ => {}
        //     }
        // }

        // if let (Some(ann), Some(elem)) = (annotation, annotated_element) {
        //     print_annotation_and_signature(&source_code, ann, elem);
        // }
        // else if let Some(ann) = annotation {
        //     println!("Annotation: {}", &source_code[ann.range().start_byte..ann.range().end_byte]);
        // }
    }
}

fn print_annotation_and_signature(source: &str, annotation: Node, element: Node) {
    let annotation_text = &source[annotation.range().start_byte..annotation.range().end_byte];
    let element_type = element.kind();
    let signature = get_element_signature(source, element);

    println!("Annotation: {}", annotation_text);
    println!("Annotated element type: {}", element_type);
    println!("Signature: {}", signature);
    println!("----");
}

fn get_element_signature(source: &str, node: Node) -> String {
    match node.kind() {
        "class_declaration" => {
            let end = node.child_by_field_name("body")
                .map_or(node.end_byte(), |body| body.start_byte());
            source[node.start_byte()..end].trim().to_string()
        },
        "method_declaration" => {
            let end = node.child_by_field_name("body")
                .map_or(node.end_byte(), |body| body.start_byte());
            source[node.start_byte()..end].trim().to_string()
        },
        "field_declaration" => {
            let end = node.child_by_field_name("declarator")
                .and_then(|d| d.child_by_field_name("value"))
                .map_or(node.end_byte(), |v| v.start_byte());
            source[node.start_byte()..end].trim().to_string()
        },
        _ => "Unknown element type".to_string(),
    }
}