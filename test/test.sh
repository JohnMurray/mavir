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
rm -rf ./build/
./gradlew clean build >/dev/null 2>&1 \
    || error "Failed to build AutoValue jar (gradle)"
popd

#--------------------------------------------------------------------------------
# Build AutoValue jar using mavir

info "Building classes using mavir..."
pushd "${DIR}/mavir_autovalue"
./gradlew clean build >/dev/null 2>&1 \
    || error "Failed to build AutoValue jar (mavir)"
popd

#--------------------------------------------------------------------------------
# Compare/Diff the compiled classes in the jars

info "Diffing the compiled classes..."
class_name_list=(
  "AutoValue_TestClass"
)

for class_name in "${class_name_list[@]}"; do
  mavir_class="${DIR}/mavir_autovalue/build/classes/java/main/com/github/johnmurray/mavir/${class_name}.class"
  refav_class="${DIR}/java_autovalue/build/classes/java/main/com/github/johnmurray/mavir/${class_name}.class"
  diff \
    <(javap -p -c -constants "$mavir_class") \
    <(javap -p -c -constants "$refav_class") \
    || error "Detected a difference in the class file for ${class_name}\n\t${mavir_class}\n\t${refav_class}"
done
info "No differences detected in the compiled classes"