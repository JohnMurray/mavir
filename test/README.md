# Functional Tests

This directory contains a set of functional tests that compare
AutoValue classes generated with `mavir` against those generated
with the original AutoValue project.

The control classes are built via a small gradle project. The `mavir`
classes are generated against the same sources, unpacked, and built
with gradle using an identical configuration to the control project.


### Running Tests

Tests are run via the `test.sh` shell script (nothing fancy).


### Adding Test Cases

0. Add new test classes in the `java_autovalue` project
   (also update the `mavir` command in `build.gradle`)
1. Ensure that an identically named symlink exists in the `mavir_autovalue` project
2. Update `test.sh` to include the classname in the decompiled-diff check logic
