# TPK for Rust &emsp; [![Build]][Build Link] [![Coverage]][Coverage Link] [![Crate]][Crate Link]

[Build]: https://img.shields.io/github/workflow/status/ltfourrier/tpk-rust/Build/master
[Build Link]: https://github.com/ltfourrier/tpk-rust/actions/workflows/main.yml
[Coverage]: https://img.shields.io/codecov/c/github/ltfourrier/tpk-rust
[Coverage Link]: https://codecov.io/gh/ltfourrier/tpk-rust
[Crate]: https://img.shields.io/crates/v/tpk
[Crate Link]: https://crates.io/crates/tpk

**Rust implementation of the [TPK format](https://github.com/ltfourrier/tpk-spec).**

---

This repository contains the work-in-progress code of a Rust implementation for the [TPK data format](https://github.com/ltfourrier/tpk-spec).

At the time of writing, the specification is not finalized, nor is this implementation fully compliant anyway. Therefore, I strongly advise to **not** use this crate, or even TPK data for that matter, for any important project.

## Usage

At the moment, only manual writing/reading of elements and entries is supported. This means that most data needs to be written and read manually.

### Element-based writing/reading

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
use tpk::{Element, Writer};

fn main() {
    // "output" is an already created `Write` implementor
    let mut writer = Writer::new(output);
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
use tpk::{Element, Reader};

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

fn main() {
    // "input" is an already created `Read` implementor
    let mut reader = Reader::new(input);

    let mut in_version = false;
    while let Ok(Some(element)) = reader.read_element() {
        if in_version {
            match element {
                Element::Marker(name) if name == "name" => {
                    print_string("version name", reader.read_element().unwrap().unwrap());
                }
                Element::Marker(name) if name == "major" => {
                    print_uint8("major version", reader.read_element().unwrap().unwrap());
                }
                Element::Marker(name) if name == "minor" => {
                    print_uint8("minor version", reader.read_element().unwrap().unwrap());
                }
                Element::Marker(name) if name == "patch" => {
                    print_uint8("patch version", reader.read_element().unwrap().unwrap());
                }
                _ => panic!("Unrecognized entry"),
            }
        } else {
            match element {
                Element::Marker(name) if name == "format" => {
                    print_string("format", reader.read_element().unwrap().unwrap());
                }
                Element::Marker(name) if name == "version" => {
                    in_version = true;
                    // Oops, we're not checking that version is a folder!
                    reader.read_element().unwrap().unwrap();
                }
                _ => panic!("Unrecognized entry"),
            };
        }
    }
}
```

Ouch, that's rough! And we're not even supporting all edge cases... We could easily panic on some valid TPK data for this format (for example, a `..` or `/` folder marker), or miss invalid data (for example, another element than a folder for `version`).

This way of writing and reading a file is called "element-mode". This is the lowest-level way of dealing with TPK data and should only be used by tools that need to manipulate raw TPK metadata. This is also the only way supported by `tpk-rust`, for now.

If your need is to casually and easily read and write data to and from TPK files, for example, it is best to wait for the tree-mode or even `serde` support to be implemented.

### Entry-based writing/reading

Let's try to write the aforementioned structure as TPK data using entry-based writing:

```rust
use tpk::{Element, Entry, Writer};

fn main() {
  // "output" is an already created `Write` implementor
  let mut writer = Writer::new(file);
    writer.write_entry(&Entry {
        name: "format".into(),
        elements: vec![Element::String("TPK".into())],
    });
    writer.write_entry(&Entry {
        name: "version".into(),
        elements: vec![Element::Folder],
    });
    writer.write_entry(&Entry {
        name: "name".into(),
        elements: vec![Element::String("First Development Release".into())],
    });
    writer.write_entry(&Entry {
        name: "major".into(),
        elements: vec![Element::UInteger8(0)],
    });
    writer.write_entry(&Entry {
        name: "minor".into(),
        elements: vec![Element::UInteger8(1)],
    });
    writer.write_entry(&Entry {
        name: "patch".into(),
        elements: vec![Element::UInteger8(0)],
    });
}
```

It's slightly less verbose, but more importantly it is more structure, which allows us to factorize the code a little bit:

```rust
use tpk::{Element, Entry, Writer};

#[inline(always)]
fn create_entry(name: &str, element: Element) -> Entry {
    Entry {
        name: name.into(),
        elements: vec![element],
    }
}

fn main() {
    // "output" is an already created `Write` implementor
    let mut writer = Writer::new(output);
    writer.write_entry(&create_entry("format", Element::String("TPK".into())));
    writer.write_entry(&create_entry("version", Element::Folder));
    writer.write_entry(&create_entry(
        "name",
        Element::String("First Development Release".into()),
    ));
    writer.write_entry(&create_entry("major", Element::UInteger8(0)));
    writer.write_entry(&create_entry("minor", Element::UInteger8(1)));
    writer.write_entry(&create_entry("patch", Element::UInteger8(0)));
}
```

Much better! As it shows, entry-based writing mode is particularly useful when we want to operate in a low-level mode, but we don't want to deal with the marker/element association ourselves and the small overhead is acceptable.

Reading using entry-based mode is a little easier as well:

```rust
#[inline(always)]
fn print_string(name: &'static str, element: &Element) {
  match element {
    Element::String(string) => println!("The {} is {}", name, string),
    _ => panic!("Expected string element, got something else"),
  };
}

#[inline(always)]
fn print_uint8(name: &'static str, element: &Element) {
  match element {
    Element::UInteger8(number) => println!("The {} is {}", name, number),
    _ => panic!("Expected unsigned integer element, got something else"),
  };
}

fn main() {
  // "input" is an already created `Read` implementor
  let mut reader = Reader::new(input);

    let mut in_version = false;
    while let Ok(Some(element)) = reader.read_entry() {
        if in_version {
            match element.name.as_str() {
                "name" => print_string("version name", &element.elements[0]),
                "major" => print_uint8("major version", &element.elements[0]),
                "minor" => print_uint8("minor version", &element.elements[0]),
                "patch" => print_uint8("patch version", &element.elements[0]),
                _ => panic!("Unrecognized entry"),
            }
        } else {
            match element.name.as_str() {
                "format" => print_string("format", &element.elements[0]),
                "version" => {
                    in_version = true;
                }
                _ => panic!("Unrecognized entry"),
            }
        }
    }
}
```

Unfortunately, this implementation is just less verbose: we're still not handling some edge cases like `..` or `/*` folders, and we still do not type-check the `version` folder entry.

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
- [x] Entry-mode reading and writing
- [x] CI/CD
  - [x] CI
  - [x] CD
- [x] Publish crate

### 0.1.x - Planned enhancements unrelated to the format

#### Prerequisites

- [ ] TPK-Rust 0.1 is released

#### To-do list

- [ ] Tree-mode reading and writing
- [ ] Serde support
- [ ] Parser hints optimizations
- [ ] Performance reports vs. other formats and parsers