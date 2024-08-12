#!/usr/bin/env bash

set -euo pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

function info() { local msg=$1;
  echo -e "\033[0;32m[INFO] $msg\033[0m"
}

function error() { local msg=$1;
  echo -e "\033[0;31m[ERROR] $msg\033[0m"
  exit 1
}

#--------------------------------------------------------------------------------
# Build AutoValue jar using gradle (our reference implementation)
info "Build the reference classes using AutoValue annotation processor..."
pushd "${DIR}/java_autovalue"
./gradlew clean build >/dev/null 2>&1 \
    || error "Failed to build AutoValue jar (gradle)"
classpath=$(./gradlew printClasspath 2>&1 | rg '^\.:')
popd

#--------------------------------------------------------------------------------
# Build AutoValue jar using mavir

# Build a release version of mavir
info "Building release version of mavir..."
cd "${DIR}/.."
cargo build --release >/dev/null 2>&1 \
    || error "Failed to build release version of mavir"
MAVIR="${DIR}/../target/release/mavir"

# Create a temporary directory and generate a source-jar with mavir
info "Generate the source-jar using mavir..."
temp_dir=$(mktemp -d)
$MAVIR \
    --file-path "${DIR}/java_autovalue/src/main/java/com/github/johnmurray/mavir/TestClass.java" \
    --output-path "${temp_dir}/out.jar" \
    >/dev/null 2>&1 \
    || error "Failed to generate source-jar using mavir"

# Compile the source-jar with javac
info "Compile the (mavir-generated) source-jar using javac..."
cd "$temp_dir"
unzip out.jar
javac \
  -cp "$classpath" \
  -d build \
  -g \
  $(find "${DIR}/java_autovalue/src/main/java" -name "*.java") \
  $(find . -name "*.java")

#--------------------------------------------------------------------------------
# Compare/Diff the compiled classes in the jars

info "Output jar unpacking and compiled code"
tree
echo "--------------------------------------------------------------------------------"

info "Diffing the compiled classes..."
class_name_list=(
  "AutoValue_TestClass"
)

for class_name in "${class_name_list[@]}"; do
  diff \
    <(javap -p -c -constants "${temp_dir}/build/com/github/johnmurray/mavir/${class_name}.class") \
    <(javap -p -c -constants "${DIR}/java_autovalue/build/classes/java/main/com/github/johnmurray/mavir/${class_name}.class") \
    || error "Detected a difference in the class file for ${class_name}"
done
info "No differences detected in the compiled classes"

info "Cleaning up..."
cd "$DIR"
rm -rf "$temp_dir"