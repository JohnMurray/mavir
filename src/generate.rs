use std::path::Path;
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use crate::parse::{ClassDeclarationState, MethodDeclarationState, ParseResult};
use crate::util;
use crate::util::StripMargin;

use log::debug;
use tempdir::TempDir;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};
use zip::DateTime;
use zip::write::SimpleFileOptions;

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("Invalid output path: {0}")]
    InvalidOutputPath(String),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Zip Error: {0}")]
    ZipError(String),
}

type Result<T> = std::result::Result<T, GenerateError>;

pub fn generate_code(parse_results: Vec<ParseResult>, output_path: &str) -> Result<()> {
    let output_path = validate_output_path(output_path)?;
    let working_dir = init_working_directory()?;
    debug!("Writing to output path: {}", output_path);

    // Generate code
    for parse_result in &parse_results {
        for class_decl in &parse_result.class_declarations {
            generate_java_file(
                working_dir.path(),
                class_decl,
                &parse_result.package_name,
                &parse_result.import_statements,
            )?;
        }
    }

    // Package the generated code into a source JAR
    let output_file = File::create(Path::new(&output_path)).unwrap();
    let walkdir = WalkDir::new(working_dir.path());
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), working_dir.path(), output_file)?;

    Ok(())
}

/// Check that the output path is valid
fn validate_output_path(output_path: &str) -> Result<String> {
    // Check if the output path ends in '.jar' or '.srcjar'
    if !output_path.ends_with(".jar") && !output_path.ends_with(".srcjar") {
        return Err(GenerateError::InvalidOutputPath(format!(
            "Output path must end in '.jar' or '.srcjar'. Got: {}", output_path)));
    }

    // If the path is a relative path, convert it to an absolute path
    let output_path = if output_path.starts_with("/") {
        output_path.to_string()
    } else {
        let current_dir = std::env::current_dir()
            .map_err(|e| GenerateError::InvalidOutputPath(format!("Failed to get current directory: {}", e)))?;
        let output_path = current_dir.join(output_path);
        output_path.to_str().unwrap().to_string()
    };

    // Check if the parent directory exists
    let parent_dir = Path::new(&output_path).parent()
        .ok_or(GenerateError::InvalidOutputPath(format!(
            "Output path must have a parent directory. Got: {}", &output_path)))?;
    if !parent_dir.exists() {
        return Err(GenerateError::InvalidOutputPath(format!(
            "Parent directory of output path does not exist. Got: {}", &output_path)));
    }

    Ok(output_path)
}

/// Create a temporary directory to write contents to. The contents of this directory
/// will be the input to the packaged source JAR. This directory will be deleted after
/// the source JAR is created.
fn init_working_directory() -> Result<TempDir> {
    let dir = TempDir::new("mavir")
        .map_err(|e| GenerateError::IoError(e))?;

    // Create a META-INF folder with a MANIFEST.MF file
    let manifest_path = dir.path().join("META-INF/MANIFEST.MF");
    fs::create_dir_all(manifest_path.parent().unwrap())
        .map_err(|e| GenerateError::IoError(e))?;
    fs::File::create(manifest_path)
        .and_then(|mut f| f.write_all(b"Manifest-Version: 1.0\nCreated-By: mavir\n"))
        .map_err(|e| GenerateError::IoError(e))?;

    Ok(dir)
}

/// Zip a directory into a provided writer given a `walkdir` iterator and a prefix path (to strip
/// from the zip file contents).
fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &Path,
    writer: T,
) -> Result<()>
where
        T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .last_modified_time(DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).unwrap());

    let prefix = Path::new(prefix);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .ok_or(GenerateError::ZipError(format!("Path is not valid UTF-8: {:?}", &name)))?;

        if path.is_file() {
            debug!("adding file {path:?} as {name:?} ...");
            zip.start_file(path_as_string, options)
                .map_err(|e| GenerateError::ZipError(format!("Failed to start file: {}", e)))?;
            let mut f = File::open(path)
                .map_err(|e| GenerateError::ZipError(format!("Failed to open file: {}", e)))?;
            f.read_to_end(&mut buffer)
                .map_err(|e| GenerateError::ZipError(format!("Failed to read file: {}", e)))?;
            zip.write_all(&buffer)
                .map_err(|e| GenerateError::ZipError(format!("Failed to write file: {}", e)))?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // If not a file (and not the root), then it must be a directory
            zip.add_directory(name.to_str().unwrap(), options)
                .map_err(|e| GenerateError::ZipError(format!("Failed to add directory: {}", e)))?;
        }
    }

    zip.finish()
        .map_err(|e| GenerateError::ZipError(format!("Failed to finish zip file: {}", e)))?;
    Ok(())
}

fn generate_java_file(
    working_dir: &Path,
    class: &ClassDeclarationState,
    package_name: &str,
    import_statements: &Vec<String>
) -> Result<()>
{
    let class_name = get_class_name(class);
    let parent_class_name = get_parent_class_name(class);

    let package_path = package_name.replace(".", "/");
    let class_path = working_dir
        .join(&package_path)
        .join(format!("{}.java", class_name));

    fs::create_dir_all(class_path.parent().unwrap())
        .map_err(|e| GenerateError::IoError(e))?;
    let mut file = fs::File::create(class_path)
        .map_err(|e| GenerateError::IoError(e))?;
    let mut writer = BufWriter::new(&mut file);
    writer.write(template_file_contents(
        &class_name,
        &parent_class_name,
        package_name,
        import_statements,
        class
    ).as_bytes()).unwrap();

    Ok(())
}

fn get_class_name(class: &ClassDeclarationState) -> String {
    // Join all strings in parent_chain with '_' and append the name
    let mut class_name = class.name.to_string();
    for parent in &class.parent_chain {
        class_name = format!("{}_{}", parent, class_name);
    }
    class_name = format!("AutoValue_{class_name}");

    debug!("Generating code for {}", &class_name);
    return class_name;
}

fn get_parent_class_name(class: &ClassDeclarationState) -> String {
    let mut class_name = class.name.to_string();
    for parent in &class.parent_chain {
        class_name = format!("{}.{}", parent, class_name);
    }
    return class_name;
}

fn template_file_contents(
    class_name: &str,
    parent_class_name: &str,
    package_name: &str,
    import_statements: &Vec<String>,
    class: &ClassDeclarationState
) -> String {
    let imports = import_statements.join("\n");

    let modifiers = &class.modifiers.join(" ");
    let instance_vars = class.methods
        .iter()
        .map(template_instance_var_decls)
        .collect::<Vec<String>>()
        .join("\n    ");
    let getters = class.methods
        .iter()
        .map(template_getter)
        .collect::<Vec<String>>()
        .join("\n");
    let constructor = template_constructor(class_name, class);
    let to_string = template_to_string(parent_class_name, class);
    let equals = template_equals(parent_class_name, class);
    let hashcode = template_hashcode(class);

    format!(r#"package {package_name};
    |
    |{imports}
    |
    |final {modifiers} class {class_name} extends {parent_class_name} {{
    |    {instance_vars}
    |    {constructor}
    |    {getters}
    |    {to_string}
    |    {equals}
    |    {hashcode}
    |}}
    |"#).strip_margin()
}

fn template_to_string(class_name: &str, class: &ClassDeclarationState) -> String {

    // The parent class-name is qualified for nested classes, such a 'OuterClass.InnerAutoValueClass'
    // But, for our toString method, we want just 'InnerAutoValueClass'
    let class_name = class_name.split(".").last().unwrap();

    let instance_vars = class.methods
        .iter()
        .map(|m| format!(r#""{}=" + this.{}"#, m.name, m.name))
        .collect::<Vec<String>>()
        .join(" + \", \"\n            + ");

    format!(r#"
    |    @Override
    |    public String toString() {{
    |        return "{class_name}{{"
    |            + {instance_vars}
    |            + "}}";
    |    }}
    |"#).strip_margin()
}

fn template_equals(parent_class_name: &str, class: &ClassDeclarationState) -> String {

    let equals_checks = class.methods
        .iter()
        .map(|m| {
            let name = &m.name;
            if &m.return_type == "double" {
                format!("Double.doubleToLongBits(this.{name}) == Double.doubleToLongBits(that.{name}())")
            }
            else if &m.return_type == "float" {
                format!("Float.floatToIntBits(this.{name}) == Float.floatToIntBits(that.{name}())")
            }
            else if util::is_primitive_type(&m.return_type) {
                format!("this.{name} == that.{name}()")
            } else {
                let if_null = if m.is_nullable {
                    &format!("this.{name} == null? that.{name}() == null : ")
                } else { "" };
                format!("({if_null}this.{name}.equals(that.{name}()))")
            }
        })
        .collect::<Vec<String>>()
        .join("\n                && ");

    format!(r#"
    |    @Override
    |    public boolean equals(Object o) {{
    |        if (o == this) {{
    |            return true;
    |        }}
    |        if (o instanceof {parent_class_name}) {{
    |            {parent_class_name} that = ({parent_class_name}) o;
    |            return {equals_checks};
    |        }}
    |        return false;
    |    }}
    |"#).strip_margin()
}

fn template_hashcode(class: &ClassDeclarationState) -> String {
    let field_hashes = class.methods
        .iter()
        .map(|m| {
            let name = &m.name;
            let mult = "h$ *= 1000003;\n       ";
            if &m.return_type == "long" {
                format!("{mult} h$ ^= (int) (({name} >>> 32) ^ {name});")
            } else if &m.return_type == "boolean" {
                format!("{mult} h$ ^= {name} ? 1231 : 1237;")
            } else if &m.return_type == "double" {
                format!("{mult} h$ ^= (int) ((Double.doubleToLongBits({name}) >>> 32) ^ Double.doubleToLongBits({name}));")
            } else if &m.return_type == "float" {
                format!("{mult} h$ ^= Float.floatToIntBits({name});")
            } else if util::is_primitive_type(&m.return_type) {
                format!("{mult} h$ ^= this.{name};")
            } else {
                let if_null =
                    if m.is_nullable { &format!("this.{name} == null ? 0 : ") }
                    else { "" };
                format!("{mult} h$ ^= {if_null}this.{name}.hashCode();")
            }
        })
        .collect::<Vec<String>>()
        .join("\n        ");


    format!(r#"
    |    @Override
    |    public int hashCode() {{
    |        int h$ = 1;
    |        {field_hashes}
    |        return h$;
    |    }}
    |"#).strip_margin()
}

fn template_constructor(class_name: &str, class: &ClassDeclarationState) -> String {
    let constructor_params = class.methods
        .iter()
        .map(|method| {
            format!("{}{} {}",
                    if method.is_nullable { "@Nullable " } else { "" },
                    method.return_type,
                    method.name)
        })
        .collect::<Vec<String>>()
        .join(",\n            ");

    // TODO: We should skip the null-check if the type is nullable
    let assignments = class.methods
        .iter()
        .map(|method| {
            let name = &method.name;
            if util::is_primitive_type(&method.return_type) || method.is_nullable {
                format!(r#"        this.{name} = {name};"#)
            } else {
                format!(r#"
                |        if ({name} == null) {{
                |            throw new NullPointerException("Null {name}");
                |        }}
                |        this.{name} = {name};
                |"#).strip_margin()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    format!(r#"
    |    {class_name}(
    |            {constructor_params}) {{
    |{assignments}
    |    }}
    |"#).strip_margin()
}

fn template_instance_var_decls(method: &MethodDeclarationState) -> String {
    let name = &method.name;
    let return_type = &method.return_type;
    let nullable_annotation = if method.is_nullable { "@Nullable " } else { "" };
    format!("{nullable_annotation}private final {return_type} {name};")
}

fn template_getter(method: &MethodDeclarationState) -> String {
    let name = &method.name;
    let return_type = &method.return_type;

    // Join all the modifiers except "abstract"
    let modifiers = method.modifiers.iter()
        .filter(|m| *m != "abstract")
        .cloned()
        .collect::<Vec<String>>()
        .join(" ");

    // TODO: Do we need to make the name different from the accessor when using getters?
    // let function_name = format!("get{}", upper_case_first_letter(name));

    format!(r#"
    |    @Override
    |    {modifiers} {return_type} {name}() {{
    |        return this.{name};
    |    }}
    |"#).strip_margin()
}

// fn upper_case_first_letter(symbol: &str) -> String {
//     let mut chars = symbol.chars();
//     match chars.next() {
//         None => String::new(),
//         Some(first_char) => first_char.to_uppercase().chain(chars).collect(),
//     }
// }