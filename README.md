# mavir

A minimal AutoValue implimentation in Rust.

```text
Usage: mavir [OPTIONS] --output-path <OUTPUT_PATH>

Options:
  -f, --file-path <FILE_PATH>      Path to a Java source file
  -o, --output-path <OUTPUT_PATH>  Path to the output file that will contain the generated code. This should
                                   be a path to a source JAR. The path MUST not exist, but the parent directory
                                   is expected to exist
  -v, --verbose                    Print Verbose output. This can also be configured with 'RUST_LOG=debug'
  -h, --help                       Print help
  -V, --version                    Print version
```

### Debugging

You can do some basic spot-checking or debugging by running the tool and inspecting
the output in the generated JAR like follows:
```shell
cargo run -- \
  --file-paths /path/to/file/with/AutoValueModels.java \
  --output-path boop.jar \
  --verbose\
  \
  && unzip -p boop.jar \
    path/to/file/with/AutoValue_AutoValueModels.java
```

### TODO's

- [x] Modifiers on accessors should match abstract methods (currently just defaults to public)
- [x] ~Modifiers on constructor shoud match something (class visibility)~
- [x] Comments are not appropriately ignored when gathering methods.
      This may be an element ordering thing
- [x] Test nested class construction
