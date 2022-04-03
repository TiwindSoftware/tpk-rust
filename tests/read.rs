use std::io::Cursor;
use tpk::read::Error;
use tpk::{Element, TpkReader};

macro_rules! read_element {
    ($i:ident reads to $p:pat => $e:expr) => {
        let cursor = Cursor::new($i);
        let mut reader = TpkReader::new(cursor);
        let result = reader.read_element().unwrap();
        match result {
            $p => $e,
            _ => panic!("Expected specific element"),
        }
    };
    ($i:ident reads to $p:pat) => {read_element!($i reads to $p => ())};
    ($i:ident fails with $p:pat => $e:expr) => {
        let cursor = Cursor::new($i);
        let mut reader = TpkReader::new(cursor);
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
    read_element!(input reads to Element::Boolean(value) => assert_eq!(value, false));
}

#[test]
fn test_read_true_boolean() {
    let input = vec![0b00110001u8];
    read_element!(input reads to Element::Boolean(value) => assert_eq!(value, true));
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
fn test_extension_not_supported() {
    let input = vec![0b01110000u8, 0b00000000u8];
    read_element!(input fails with Error::UnsupportedType(pos, msg) => {
        assert_eq!(pos, 0);
        assert_eq!(msg, "extension");
    });
}
