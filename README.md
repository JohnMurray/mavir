# MAViR

A minimal AutoValue implimentation in Rust.

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

- [ ] Modifiers on accessors should match abstract methods (currently just defaults to public)
- [ ] Modifiers on constructor shoud match something (class visibility)
- [x] Comments are not appropriately ignored when gathering methods.
      This may be an element ordering thing
