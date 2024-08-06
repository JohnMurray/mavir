use std::path::Path;
use std::io::Write;
use std::fs;

use crate::parse::ParseResult;

use log::{debug, warn};
use tempdir::TempDir;

#[derive(Debug)]
pub enum GenerateError {
    InvalidOutputPath(String),
    IoError(std::io::Error),
}

type Result<T> = std::result::Result<T, GenerateError>;

pub fn generate_code(parse_result: ParseResult, output_path: &str) -> Result<()> {
    let output_path = validate_output_path(output_path)?;
    debug!("Writing to output path: {}", output_path);
    // TODO: Generate code
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