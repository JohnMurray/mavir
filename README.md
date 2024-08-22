# mavir

Mavir is a minimal code-generation replacement for AutoValue that aims to generate byte-for-byte compatible
classes, but without relying on the annotation processor for generating code.

### Features

The key to note here is "minimal". Mavir does not aim to provide full compatibility with AutoValue, but does
aim to cover the most basic features. Currently supported features:

- `@AutoValue` annotation in both top-level and nested contexts
- Using bean-style getters for the abstract class.
- `@Nullable` annotation (removes null checks from constructor)

Features currently unavailable but currently under development:

- Builder support (`@AutoValue.Builder`)
- Optional support in builders (optionals currently work like any other type in base AutoValue class)

Features that are unavailable and ulikely to be supported:

- custom `equals`, `hashCode`, or `toString` implementation(s)
- "Passing through" annotations to the generated impl.
- Extensions
- Pretty much everything else in the AutoValue docs...

### Why?

The Java annotation processor is perfectly suitable if you always want to compile your code when you perform
code-generation. For building, packagaing, or testing an application; this is likely fine. This is because the
annotation processor runs as a compiler plugina and requires the full context of a compilation, meaning that you
don't just need to build the current package you want to run AutoValue code-generation on, but you need to build all
the transitive dependencies to that package as well.

There are instances where you want to generate the code without invoking the compiler (e.g. performing code-generation
for an IDE). Generating code outside the annotation processor means skipping _any_ compilation of Java code.


### How?

Mavir works by parsing Java files and scanning the AST to extract information on AutoValue classes and then generating
class files and packaging them into a jar.

Mavir's generated artifacts are consistent between runs and suitable for use in a hermetic build system.

### Help

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
- [ ] Support getters and setters
      - [x] Support `getField` for auto-value classes
      - [ ] Support `setField` for builder classes
- [x] Support `@Nullable` values
- [x] Support `Optional` values in AutoValue
- [ ] Support `Optional` values in AutoValue.Builder
      - Do we need to support Guava Optional as well?
        https://github.com/google/auto/blob/main/value/userguide/builders-howto.md#optional
- [ ]
