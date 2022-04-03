use std::iter::repeat;

use tpk::Element;

fn assert_element_write(element: Element, expected_size: usize) -> Vec<u8> {
    let mut output = vec![];
    element.write(&mut output).unwrap();
    assert_eq!(output.len(), expected_size);
    output
}

#[test]
fn test_write_marker() {
    let output = assert_element_write(Element::Marker(String::from("test")), 5);
    assert_eq!(output[0], 0b10000100u8);
    assert_eq!(&output[1..5], b"test".as_slice());
}

#[test]
fn test_write_marker_with_long_name() {
    let name = String::from_iter(repeat('a').take(987654));
    let output = assert_element_write(Element::Marker(name), 987657);
    assert_eq!(output[..3], vec![0b11000110u8, 0b11001000u8, 0b01111000u8]);
    assert_eq!(&output[3..987657], vec![b'a'; 987654].as_slice());
}

#[test]
fn test_write_folder() {
    let output = assert_element_write(Element::Folder, 1);
    assert_eq!(output[0], 0u8);
}

#[test]
fn test_write_collection() {
    let output = assert_element_write(Element::Collection, 1);
    assert_eq!(output[0], 1u8);
}

#[test]
fn test_write_uint8() {
    let output = assert_element_write(Element::UInteger8(42), 2);
    assert_eq!(output[0], 0b00100000u8);
    assert_eq!(output[1], 0b00101010u8);
}

#[test]
fn test_write_uint16() {
    let output = assert_element_write(Element::UInteger16(1337), 3);
    assert_eq!(output[0], 0b00100001u8);
    assert_eq!(output[1..], vec![0b00111001u8, 0b00000101u8]);
}

#[test]
fn test_write_uint32() {
    let output = assert_element_write(Element::UInteger32(22101995), 5);
    assert_eq!(output[0], 0b00100010u8);
    assert_eq!(
        output[1..],
        vec![0b11101011u8, 0b00111111u8, 0b01010001u8, 0b00000001u8]
    );
}

#[test]
fn test_write_uint64() {
    let output = assert_element_write(Element::UInteger64(987654321123456789), 9);
    assert_eq!(output[0], 0b00100011u8);
    assert_eq!(
        output[1..],
        vec![
            0b00010101u8,
            0b01110111u8,
            0b01110001u8,
            0b01001011u8,
            0b01011111u8,
            0b11011010u8,
            0b10110100u8,
            0b00001101u8
        ]
    );
}

#[test]
fn test_write_int8() {
    let output = assert_element_write(Element::Integer8(-42), 2);
    assert_eq!(output[0], 0b00100100u8);
    assert_eq!(output[1], 0b11010110u8);
}

#[test]
fn test_write_int16() {
    let output = assert_element_write(Element::Integer16(-1337), 3);
    assert_eq!(output[0], 0b00100101u8);
    assert_eq!(output[1..], vec![0b11000111u8, 0b11111010u8]);
}

#[test]
fn test_write_int32() {
    let output = assert_element_write(Element::Integer32(-22101995), 5);
    assert_eq!(output[0], 0b00100110u8);
    assert_eq!(
        output[1..],
        vec![0b00010101u8, 0b11000000u8, 0b10101110u8, 0b11111110u8]
    );
}

#[test]
fn test_write_int64() {
    let output = assert_element_write(Element::Integer64(-987654321123456789), 9);
    assert_eq!(output[0], 0b00100111u8);
    assert_eq!(
        output[1..],
        vec![
            0b11101011u8,
            0b10001000u8,
            0b10001110u8,
            0b10110100u8,
            0b10100000u8,
            0b00100101u8,
            0b01001011u8,
            0b11110010u8
        ]
    );
}

#[test]
fn test_write_float32() {
    let output = assert_element_write(Element::Float32(-738.775), 5);
    assert_eq!(output[0], 0b00101110u8);
    assert_eq!(
        output[1..],
        vec![0b10011010u8, 0b10110001u8, 0b00111000u8, 0b11000100u8]
    );
}

#[test]
#[allow(clippy::excessive_precision)]
fn test_write_float64() {
    let output = assert_element_write(Element::Float64(-45743210879.52954864501953125), 9);
    assert_eq!(output[0], 0b00101111u8);
    assert_eq!(
        output[1..],
        vec![
            0b00100001u8,
            0b00001111u8,
            0b11111111u8,
            0b00000010u8,
            0b00000100u8,
            0b01001101u8,
            0b00100101u8,
            0b11000010u8
        ]
    );
}

#[test]
fn test_write_boolean_false() {
    let output = assert_element_write(Element::Boolean(false), 1);
    assert_eq!(output[0], 0b00110000u8);
}

#[test]
fn test_write_boolean_true() {
    let output = assert_element_write(Element::Boolean(true), 1);
    assert_eq!(output[0], 0b00110001u8);
}

#[test]
fn test_write_small_string() {
    let string = "This is a fairly short string, under 255 characters.";
    let output = assert_element_write(Element::String(String::from(string)), 54);
    assert_eq!(output[0], 0b00010000u8);
    assert_eq!(output[1], 52u8);
    assert_eq!(&output[2..], string.as_bytes());
}

#[test]
fn test_write_medium_string() {
    let output = assert_element_write(Element::String(String::from_iter(vec!['a'; 500])), 503);
    assert_eq!(output[0], 0b00010001u8);
    assert_eq!(output[1..3], vec![0b11110100u8, 0b00000001u8]);
    assert_eq!(output[3..], vec![b'a'; 500]);
}

#[test]
fn test_write_small_blob() {
    let output = assert_element_write(Element::Blob(vec![42u8; 50]), 52);
    assert_eq!(output[0], 0b00010100u8);
    assert_eq!(output[1], 50u8);
    assert_eq!(&output[2..], vec![42u8; 50]);
}

#[test]
fn test_write_medium_blob() {
    let output = assert_element_write(Element::Blob(vec![42u8; 500]), 503);
    assert_eq!(output[0], 0b00010101u8);
    assert_eq!(output[1..3], vec![0b11110100u8, 0b00000001u8]);
    assert_eq!(output[3..], vec![42u8; 500]);
}
