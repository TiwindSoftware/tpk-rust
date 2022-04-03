use crate::read::Error::{Syntax, UnknownType, UnsupportedType};
use crate::Element;
use byteorder::{ByteOrder, LE};
use std::{io, string};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unknown error")]
    Unknown,

    #[error("I/O error while reading TPK data: {source}")]
    Io {
        #[from]
        source: io::Error,
    },

    #[error("Syntax error at byte {0}: {1}")]
    Syntax(usize, &'static str),

    #[error("Unknown element type at byte {0}: {1:#X}")]
    UnknownType(usize, u8),

    #[error("Invalid UTF-8 character at byte {pos}: {source}")]
    InvalidString {
        pos: usize,

        #[source]
        source: string::FromUtf8Error,
    },

    #[error("Unsupported element type at byte {0}: {1}")]
    UnsupportedType(usize, &'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct TpkReader<T> {
    read: T,
    previous_bytes_read: usize,
    bytes_read: usize,
}

const UNEXPECTED_EOF: &str = "expected more, got EOF";

impl<T> TpkReader<T>
where
    T: io::Read,
{
    pub fn new(read: T) -> TpkReader<T> {
        TpkReader {
            read,
            previous_bytes_read: 0,
            bytes_read: 0,
        }
    }

    pub fn read_element(&mut self) -> Result<Element> {
        let type_byte = self.expect::<1>()?[0];
        if type_byte & 0b10000000 != 0 {
            return self.read_marker(type_byte);
        }

        match (type_byte & 0xF0) >> 4 {
            0b0000 => self.read_folder(type_byte),
            0b0010 => self.read_number(type_byte),
            0b0011 => self.read_boolean(type_byte),
            0b0001 => self.read_string_or_blob(type_byte),
            0b0111 => Err(UnsupportedType(self.previous_bytes_read, "extension")),
            _ => Err(UnknownType(self.previous_bytes_read, type_byte)),
        }
    }

    fn read_marker(&mut self, type_byte: u8) -> Result<Element> {
        let mut has_more = type_byte & 0b01000000 != 0;
        let mut size = (type_byte & 0b111111) as usize;
        let mut shift = 6;
        while has_more {
            let byte = self.expect::<1>()?[0];
            has_more = byte & 0b10000000 != 0;
            size |= ((byte & 0b01111111) as usize) << shift;
            shift += 7;
        }

        self.read_utf8_string(size)
            .map(|string| Element::Marker(string))
    }

    fn read_folder(&mut self, type_byte: u8) -> Result<Element> {
        match type_byte {
            0 => Ok(Element::Folder),
            1 => Ok(Element::Collection),
            _ => Err(UnknownType(self.previous_bytes_read, type_byte)),
        }
    }

    fn read_number(&mut self, type_byte: u8) -> Result<Element> {
        match type_byte {
            0b00100000 => Ok(Element::UInteger8(self.expect::<1>()?[0])),
            0b00100001 => Ok(Element::UInteger16(LE::read_u16(
                self.expect::<2>()?.as_slice(),
            ))),
            0b00100010 => Ok(Element::UInteger32(LE::read_u32(
                self.expect::<4>()?.as_slice(),
            ))),
            0b00100011 => Ok(Element::UInteger64(LE::read_u64(
                self.expect::<8>()?.as_slice(),
            ))),
            0b00100100 => Ok(Element::Integer8(self.expect::<1>()?[0] as i8)),
            0b00100101 => Ok(Element::Integer16(LE::read_i16(
                self.expect::<2>()?.as_slice(),
            ))),
            0b00100110 => Ok(Element::Integer32(LE::read_i32(
                self.expect::<4>()?.as_slice(),
            ))),
            0b00100111 => Ok(Element::Integer64(LE::read_i64(
                self.expect::<8>()?.as_slice(),
            ))),
            0b00101110 => Ok(Element::Float32(LE::read_f32(
                self.expect::<4>()?.as_slice(),
            ))),
            0b00101111 => Ok(Element::Float64(LE::read_f64(
                self.expect::<8>()?.as_slice(),
            ))),
            _ => Err(UnknownType(self.previous_bytes_read, type_byte)),
        }
    }

    fn read_boolean(&mut self, type_byte: u8) -> Result<Element> {
        match type_byte {
            0b00110000 => Ok(Element::Boolean(false)),
            0b00110001 => Ok(Element::Boolean(true)),
            _ => Err(UnknownType(self.previous_bytes_read, type_byte)),
        }
    }

    fn read_string_or_blob(&mut self, type_byte: u8) -> Result<Element> {
        // We need to store this because "read_bundled_size" ALSO reads the size bytes, so the
        // position in the error is wrong if the sub type is invalid.
        let previous_bytes_read = self.previous_bytes_read;

        let sub_type_byte = type_byte & 0b1100;
        let size = self.read_bundled_size(type_byte)?;

        match sub_type_byte {
            0b0000 => self
                .read_utf8_string(size)
                .map(|string| Element::String(string)),
            0b0100 => self.expect_heap(size).map(|bytes| Element::Blob(bytes)),
            _ => Err(UnknownType(previous_bytes_read, type_byte)),
        }
    }

    #[inline]
    fn read_utf8_string(&mut self, size: usize) -> Result<String> {
        let string_bytes = self.expect_heap(size)?;
        String::from_utf8(string_bytes).map_err(|e| Error::InvalidString {
            pos: self.previous_bytes_read + e.utf8_error().valid_up_to(),
            source: e,
        })
    }

    #[inline]
    fn read_bundled_size(&mut self, type_byte: u8) -> Result<usize> {
        match type_byte & 0b11 {
            0b00 => Ok(self.expect::<1>()?[0] as usize),
            0b01 => Ok(LE::read_u16(self.expect::<2>()?.as_slice()) as usize),
            0b10 => Ok(LE::read_u32(self.expect::<4>()?.as_slice()) as usize),
            0b11 => Ok(LE::read_u64(self.expect::<8>()?.as_slice()) as usize),
            _ => Err(UnknownType(self.previous_bytes_read, type_byte)),
        }
    }

    fn expect<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut buf = [0u8; N];
        let bytes_read = self.read.read(&mut buf)?;
        self.previous_bytes_read = self.bytes_read;
        self.bytes_read += bytes_read;
        if bytes_read != N {
            return Err(Syntax(self.bytes_read, UNEXPECTED_EOF));
        }
        Ok(buf)
    }

    fn expect_heap(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        let bytes_read = self.read.read(&mut buf)?;
        self.previous_bytes_read = self.bytes_read;
        self.bytes_read += bytes_read;
        if bytes_read != count {
            return Err(Syntax(self.bytes_read, UNEXPECTED_EOF));
        }
        Ok(buf)
    }
}
