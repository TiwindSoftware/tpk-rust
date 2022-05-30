use std::io::Cursor;
use tpk::read::{Error, Result};
use tpk::{Element, Entry, Reader};

macro_rules! read_element {
    ($i:ident reads to $p:pat => $e:expr) => {
        let cursor = Cursor::new($i);
        let mut reader = Reader::new(cursor);
        let result = reader.read_element().unwrap();
        match result {
            Some($p) => $e,
            Some(_) => panic!("Expected specific element"),
            None => panic!("Expected some element"),
        }
    };
    ($i:ident reads to $p:pat) => {read_element!($i reads to $p => ())};
    ($i:ident fails with $p:pat => $e:expr) => {
        let cursor = Cursor::new($i);
        let mut reader = Reader::new(cursor);
        let result = reader.read_element();
        match result {
            Err(e) => {
                match e {
                    $p => $e,
                    _ => panic!("Expected specific error"),
                }
            }
            Ok(_) => panic!("Expected read to fail"),
        }
    };
}

fn read_entry(input: &[u8]) -> Result<Option<Entry>> {
    let cursor = Cursor::new(input);
    let mut reader = Reader::new(cursor);
    reader.read_entry()
}

#[test]
fn test_read_marker() {
    let input = vec![0b10000100u8, b't', b'e', b's', b't'];
    read_element!(input reads to Element::Marker(name) => assert_eq!(name, "test"));
}

#[test]
fn test_read_marker_with_long_name() {
    let mut name_bytes = vec![b'a'; 987654];
    let expected_name = String::from_utf8(name_bytes.clone()).unwrap();
    let mut input = vec![0b11000110u8, 0b11001000u8, 0b01111000u8];
    input.append(&mut name_bytes);

    read_element!(input reads to Element::Marker(name) => assert_eq!(name, expected_name));
}

#[test]
fn test_read_marker_with_missing_bytes() {
    let input = vec![0b10000100u8, b't', b'e'];
    read_element!(input fails with Error::Syntax(pos, msg) => {
        assert_eq!(pos, 3);
        assert_eq!(msg, "expected more, got EOF")
    });
}

#[test]
fn test_read_marker_with_invalid_utf8() {
    let input = vec![0b10000100u8, b't', 0xFFu8, 0xDEu8, 0xADu8];
    read_element!(input fails with Error::InvalidString {pos, ..} => assert_eq!(pos, 2));
}

#[test]
fn test_read_folder() {
    let input = vec![0b00000000u8];
    read_element!(input reads to Element::Folder);
}

#[test]
fn test_read_collection() {
    let input = vec![0b00000001u8];
    read_element!(input reads to Element::Collection);
}

#[test]
fn test_read_invalid_folder_collection_sub_type() {
    let input = vec![0b00000100u8];
    read_element!(input fails with Error::UnknownType(pos, ..) => assert_eq!(pos, 0));
}

#[test]
fn test_read_false_boolean() {
    let input = vec![0b00110000u8];
    read_element!(input reads to Element::Boolean(value) => assert!(!value));
}

#[test]
fn test_read_true_boolean() {
    let input = vec![0b00110001u8];
    read_element!(input reads to Element::Boolean(value) => assert!(value));
}

#[test]
fn test_read_invalid_boolean() {
    let input = vec![0b00110010u8];
    read_element!(input fails with Error::UnknownType(pos, ..) => assert_eq!(pos, 0));
}

#[test]
fn test_read_small_string() {
    let expected_string = "This is a fairly short string, under 255 characters.";
    let mut input = vec![0b00010000u8, 0b00110100u8];
    input.extend_from_slice(expected_string.as_bytes());
    read_element!(input reads to Element::String(string) => assert_eq!(string, expected_string));
}

#[test]
fn test_read_medium_string() {
    let mut string_bytes = vec![b'a'; 987654];
    let expected_string = String::from_utf8(string_bytes.clone()).unwrap();
    let mut input = vec![
        0b00010010u8,
        0b00000110u8,
        0b00010010u8,
        0b00001111u8,
        0b00000000u8,
    ];
    input.append(&mut string_bytes);

    read_element!(input reads to Element::String(string) => assert_eq!(string, expected_string));
}

#[test]
fn test_read_empty_string() {
    let input = vec![0b00010000u8, 0b00000000u8];
    read_element!(input reads to Element::String(string) => assert_eq!(string, ""));
}

#[test]
fn test_read_string_with_wrong_length_size() {
    let input = vec![0b00010011u8, 0b00000001u8, b't'];
    read_element!(input fails with Error::Syntax(pos, msg) => {
        assert_eq!(pos, 3);
        assert_eq!(msg, "expected more, got EOF");
    });
}

#[test]
fn test_read_string_with_wrong_length() {
    let input = vec![0b00010000u8, 0b00000100u8, b't', b'e'];
    read_element!(input fails with Error::Syntax(pos, msg) => {
        assert_eq!(pos, 4);
        assert_eq!(msg, "expected more, got EOF");
    });
}

#[test]
fn test_read_small_blob() {
    let expected_value = vec![1u8, 2u8, 3u8, 42u8];
    let mut input = vec![0b00010100u8, 0b00000100u8];
    input.extend(&expected_value);

    read_element!(input reads to Element::Blob(value) => assert_eq!(value, expected_value));
}

#[test]
fn test_read_medium_blob() {
    let expected_value = vec![42u8; 987654];
    let mut input = vec![
        0b00010110u8,
        0b00000110u8,
        0b00010010u8,
        0b00001111u8,
        0b00000000u8,
    ];
    input.extend(&expected_value);

    read_element!(input reads to Element::Blob(value) => assert_eq!(value, expected_value));
}

#[test]
fn test_read_blob_with_wrong_length_size() {
    let input = vec![0b00010111u8, 0b00000001u8, 42u8];
    read_element!(input fails with Error::Syntax(pos, msg) => {
        assert_eq!(pos, 3);
        assert_eq!(msg, "expected more, got EOF");
    });
}

#[test]
fn test_read_blob_with_wrong_length() {
    let input = vec![0b00010100u8, 0b00000100u8, 1u8, 42u8];
    read_element!(input fails with Error::Syntax(pos, msg) => {
        assert_eq!(pos, 4);
        assert_eq!(msg, "expected more, got EOF");
    });
}

#[test]
fn test_read_string_blob_with_invalid_type_byte() {
    let input = vec![0b00011000u8, 0b00000000u8];
    read_element!(input fails with Error::UnknownType(pos, ..) => assert_eq!(pos, 0));
}

#[test]
#[allow(deprecated)]
fn test_extension_not_supported() {
    let input = vec![0b01110000u8, 0b00000000u8];
    read_element!(input fails with Error::UnsupportedType(pos, msg) => {
        assert_eq!(pos, 0);
        assert_eq!(msg, "extension");
    });
}

// The following could be used to represent a timestamp while retaining the information about
// what it represents: a unix timestamp. This shows the rationale behind allowing multiple
// elements per named entry in TPK payloads.
const TIMESTAMP_ENTRY: [u8; 21] = [
    // Marker - "name"
    0b10000100u8,
    b'n',
    b'a',
    b'm',
    b'e',
    // Unsigned integer - 1651906455
    0b00100010u8,
    0b10010111u8,
    0b00010111u8,
    0b01110110u8,
    0b01100010u8,
    // String - "unix_time"
    0b00010000u8,
    0b00001001u8,
    b'u',
    b'n',
    b'i',
    b'x',
    b'_',
    b't',
    b'i',
    b'm',
    b'e',
];

#[test]
fn test_read_entry() {
    let mut input = Vec::new();
    input.extend_from_slice(&TIMESTAMP_ENTRY);

    let result = read_entry(&input).unwrap().unwrap();

    assert_eq!(result.name, "name");
    assert_eq!(result.elements.len(), 2);
    assert!(matches!(
        result.elements.get(0),
        Some(Element::UInteger32(1651906455))
    ));
    assert!(matches!(
        result.elements.get(1),
        Some(Element::String(str)) if str == "unix_time"
    ));
}

#[test]
fn test_read_entry_with_error() {
    let mut input = Vec::new();
    input.extend_from_slice(&TIMESTAMP_ENTRY);
    input[10] = 0b01000000u8;

    let result = read_entry(&input);

    assert!(matches!(result, Err(Error::UnknownType(10, 0b01000000u8))));
}

#[test]
fn test_read_two_entries() {
    let mut input = Vec::new();
    input.extend_from_slice(&TIMESTAMP_ENTRY);
    input.extend_from_slice(&TIMESTAMP_ENTRY);
    input[22] = b'l';

    // We cannot use our utility function here as we need to keep the reader's state between reads.
    let cursor = Cursor::new(input);
    let mut reader = Reader::new(cursor);
    let result = reader.read_entry().unwrap().unwrap();
    let second_result = reader.read_entry().unwrap().unwrap();

    assert_eq!(result.name, "name");
    assert_eq!(result.elements.len(), 2);
    assert!(matches!(
        result.elements.get(0),
        Some(Element::UInteger32(1651906455))
    ));
    assert!(matches!(
        result.elements.get(1),
        Some(Element::String(str)) if str == "unix_time"
    ));
    assert_eq!(second_result.name, "lame");
    assert_eq!(second_result.elements.len(), 2);
    assert!(matches!(
        second_result.elements.get(0),
        Some(Element::UInteger32(1651906455))
    ));
    assert!(matches!(
        second_result.elements.get(1),
        Some(Element::String(str)) if str == "unix_time"
    ));
}

#[test]
fn test_read_implicit_entry() {
    let mut input = Vec::new();
    input.extend_from_slice(&TIMESTAMP_ENTRY);

    let result = read_entry(&input[5..]).unwrap().unwrap();

    assert_eq!(result.name, "/");
    assert_eq!(result.elements.len(), 2);
    assert!(matches!(
        result.elements.get(0),
        Some(Element::UInteger32(1651906455))
    ));
    assert!(matches!(
        result.elements.get(1),
        Some(Element::String(str)) if str == "unix_time"
    ));
}

#[test]
fn test_read_half_consumed_entry() {
    let mut input = Vec::new();
    input.extend_from_slice(&TIMESTAMP_ENTRY);

    let cursor = Cursor::new(input);
    let mut reader = Reader::new(cursor);
    reader.read_element().unwrap(); // Read the marker
    reader.read_element().unwrap(); // Read the integer
    let result = reader.read_entry().unwrap().unwrap();

    assert_eq!(result.name, "name");
    assert_eq!(result.elements.len(), 1);
    assert!(matches!(
        result.elements.get(0),
        Some(Element::String(str)) if str == "unix_time"
    ));
}
