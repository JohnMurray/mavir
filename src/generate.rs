use std::path::Path;
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use crate::parse::ParseResult;

use log::{debug, warn};
use tempdir::TempDir;
use walkdir::{DirEntry, WalkDir};
use zip::DateTime;
use zip::write::SimpleFileOptions;

#[derive(Debug)]
pub enum GenerateError {
    InvalidOutputPath(String),
    IoError(std::io::Error),
    ZipError(String),
}

type Result<T> = std::result::Result<T, GenerateError>;

pub fn generate_code(parse_result: ParseResult, output_path: &str) -> Result<()> {
    let output_path = validate_output_path(output_path)?;
    let working_dir = init_working_directory()?;
    debug!("Writing to output path: {}", output_path);

    // TODO: Generate code

    // TODO: Package the generated code into a source JAR
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