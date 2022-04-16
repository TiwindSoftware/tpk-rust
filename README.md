# TPK for Rust &emsp; [![Build]][Build Link] [![Coverage]][Coverage Link]

[Build]: https://github.com/ltfourrier/tpk-rust/actions/workflows/main.yml/badge.svg
[Build Link]: https://github.com/ltfourrier/tpk-rust/actions/workflows/main.yml
[Coverage]: https://codecov.io/gh/ltfourrier/tpk-rust/branch/master/graph/badge.svg?token=KKL1XJS3NU
[Coverage Link]: https://codecov.io/gh/ltfourrier/tpk-rust

**Rust implementation of the [TPK format](https://github.com/ltfourrier/tpk-spec).**

---

This repository contains the work-in-progress code of a Rust implementation for the [TPK data format](https://github.com/ltfourrier/tpk-spec).

At the time of writing, the specification is not finalized, nor is this implementation fully compliant anyway. Therefore, I strongly advise to **not** use this crate, or even TPK data for that matter, for any important project.

## Usage

At the moment, only manual writing/reading of elements is supported. This means that markers, folders and elements all need to be written and read manually.

For example, to write the TPK equivalent of the following JSON structure:

```json
{
  "format": "TPK",
  "version": {
    "name": "First Development Release",
    "major": 0,
    "minor": 1,
    "patch": 0
  }
}
```

you would need to do the following:

```rust
use tpk::{Element, TpkWriter};

fn main() {
    // "output" is an already created `Write` implementor
    let mut writer = TpkWriter::new(output);
    writer.write_element(&Element::Marker("format".into()));
    writer.write_element(&Element::String("TPK".into()));
    writer.write_element(&Element::Marker("version".into()));
    writer.write_element(&Element::Folder);
    writer.write_element(&Element::Marker("name".into()));
    writer.write_element(&Element::String("First Development Release".into()));
    writer.write_element(&Element::Marker("major".into()));
    writer.write_element(&Element::UInteger8(0));
    writer.write_element(&Element::Marker("minor".into()));
    writer.write_element(&Element::UInteger8(1));
    writer.write_element(&Element::Marker("patch".into()));
    writer.write_element(&Element::UInteger8(0));
}
```

This looks quite verbose. Reading is even worse:

```rust
use tpk::{Element, TpkReader};

fn main() {
    // "input" is an already created `Read` implementor
    let mut reader = TpkReader::new(input);

    // At the moment end of file = syntax error, so let's treat errors as EOF
    let mut in_version = false;
    while let Ok(element) = reader.read_element() {
        if in_version {
            match element {
                Element::Marker(name) if name == "name" => {
                    print_string("version name", reader.read_element().unwrap());
                }
                Element::Marker(name) if name == "major" => {
                    print_uint8("major version", reader.read_element().unwrap());
                }
                Element::Marker(name) if name == "minor" => {
                    print_uint8("minor version", reader.read_element().unwrap());
                }
                Element::Marker(name) if name == "patch" => {
                    print_uint8("patch version", reader.read_element().unwrap());
                }
                _ => panic!("Unrecognized entry"),
            }
        } else {
            match element {
                Element::Marker(name) if name == "format" => {
                    print_string("format", reader.read_element().unwrap());
                }
                Element::Marker(name) if name == "version" => {
                    in_version = true;
                    // Oops, we're not checking that version is a folder!
                    reader.read_element().unwrap();
                }
                _ => panic!("Unrecognized entry"),
            };
        }
    }
}

#[inline(always)]
fn print_string(name: &'static str, element: Element) {
    match element {
        Element::String(string) => println!("The {} is {}", name, string),
        _ => panic!("Expected string element, got something else"),
    };
}

#[inline(always)]
fn print_uint8(name: &'static str, element: Element) {
    match element {
        Element::UInteger8(number) => println!("The {} is {}", name, number),
        _ => panic!("Expected unsigned integer element, got something else"),
    };
}
```

Ouch, that's rough! And we're not even supporting all edge cases... We could easily panic on some valid TPK data for this format (for example, a `..` or `/` folder marker), or miss invalid data (for example, another element than a folder for `version`).

This way of writing and reading a file is called "element-mode". This is the lowest-level way of dealing with TPK data and should only be used by tools that need to manipulate raw TPK metadata. This is also the only way supported by `tpk-rust`, for now.

If your need is to casually and easily read and write data to and from TPK files, for example, it is best to wait for the entry-mode, tree-mode or even `serde` support to be implemented.

## Roadmap

Since `tpk-rust` is planned to be the reference implementation for the TPK data format, major and minor releases will follow those of the specification.

### 0.1 - First Development Release

#### Prerequisites

- [ ] TPK 0.1 is released

#### To-do list

- [ ] Full compliance with the specification for both write/read
  - [x] Marker TPK elements
  - [x] Primitive TPK elements
  - [x] Folders and collections
  - [ ] Extension elements
  - [ ] Dependency management
  - [ ] Big endianness support
  - [ ] Parser hints (e.g. data size)
- [ ] Entry-mode reading and writing
- [ ] CI/CD
  - [x] CI
  - [ ] CD
- [ ] Publish crate

### 0.1.x - Planned enhancements unrelated to the format

#### Prerequisites

- [ ] TPK-Rust 0.1 is released

#### To-do list

- [ ] Tree-mode reading and writing
- [ ] Serde support
- [ ] Parser hints optimizations
- [ ] Performance reports vs. other formats and parsers