use crate::model::Entry;
use crate::read::Error::{Syntax, UnknownType};
use crate::Element;
use byteorder::{ByteOrder, LE};
use std::{io, string};
use thiserror::Error;

/// Representation of a TPK read error.
#[derive(Error, Debug)]
pub enum Error {
    /// An unknown error happened.
    ///
    /// This error is "technical unknown", it should only be used in cases where the user is not
    /// supposed to get an error but gets one anyway. For example, this error should *never* be
    /// thrown for a problem with a TPK file. More simply put, this error being returned anywhere
    /// should be considered a bug or a feature that is not yet implemented.
    #[error("Unknown error")]
    Unknown,

    /// A I/O error happened.
    #[error("I/O error while reading TPK data: {source}")]
    Io {
        #[from]
        source: io::Error,
    },

    /// The end of file has been reached.
    ///
    /// Note that this error can be considered normal behavior,
    #[error("End of file reached")]
    Eof,

    /// A syntax error happened.
    ///
    /// This error happens when the TPK payload that is being read is corrupted or invalid.
    #[error("Syntax error at byte {0}: {1}")]
    Syntax(usize, &'static str),

    /// A type is unknown.
    ///
    /// This error happens when the TPK payload that is being read is lexically valid, but an
    /// unknown type byte has been encountered.
    #[error("Unknown element type at byte {0}: {1:#X}")]
    UnknownType(usize, u8),

    /// A UTF-8 string is invalid.
    ///
    /// This error happens when the TPK payload that is being read contains an invalid UTF-8
    /// character at a place where it should be expected.
    #[error("Invalid UTF-8 character at byte {pos}: {source}")]
    InvalidString {
        pos: usize,

        #[source]
        source: string::FromUtf8Error,
    },

    /// A type is unsupported.
    ///
    /// This error happens when the TPK payload that is being read is both lexically and
    /// semantically valid, but an unsupported type byte has been encountered.
    ///
    /// Note that the mere existence of this error makes this crate non-TPK-compliant, and as such
    /// this error case should be expected to be removed in the near future.
    #[deprecated]
    #[error("Unsupported element type at byte {0}: {1}")]
    UnsupportedType(usize, &'static str),
}

/// Representation of a TPK read result.
pub type Result<T> = std::result::Result<T, Error>;

/// A TPK reader structure.
///
/// This structure holds the source from which TPK data should be read, as well as internal reader
/// contextual data.
pub struct Reader<T> {
    read: T,
    previous_bytes_read: usize,
    bytes_read: usize,
    current_name: String,
    retained_element: Option<Element>,
}

const UNEXPECTED_EOF: &str = "expected more, got EOF";

impl<T> Reader<T>
where
    T: io::Read,
{
    /// Create a new [TPK reader][Reader].
    pub fn new(read: T) -> Reader<T> {
        Reader {
            read,
            previous_bytes_read: 0,
            bytes_read: 0,
            current_name: String::from("/"),
            retained_element: None,
        }
    }

    /// Read an [element][Element] from this reader.
    ///
    /// This function will consume bytes from the source reader, and will attempt to parse them
    /// and construct a new [element][Element].
    pub fn read_element(&mut self) -> Result<Option<Element>> {
        if let Some(retained_element) = self.retained_element.take() {
            return Ok(Some(retained_element));
        }

        let mut type_byte_buf = [0u8; 1];
        let bytes_read = self.read.read(&mut type_byte_buf)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        self.previous_bytes_read = self.bytes_read;
        self.bytes_read += bytes_read;
        let type_byte = type_byte_buf[0];
        if type_byte & 0b10000000 != 0 {
            let element = self.read_marker(type_byte)?;
            return Ok(Some(element));
        }

        #[allow(deprecated)]
        let element = match (type_byte & 0xF0) >> 4 {
            0b0000 => self.read_folder(type_byte),
            0b0010 => self.read_number(type_byte),
            0b0011 => self.read_boolean(type_byte),
            0b0001 => self.read_string_or_blob(type_byte),
            0b0111 => Err(Error::UnsupportedType(
                self.previous_bytes_read,
                "extension",
            )),
            _ => Err(UnknownType(self.previous_bytes_read, type_byte)),
        }?;
        Ok(Some(element))
    }

    /// Read an [entry][Entry] from this reader.
    ///
    /// Reading an entry means reading one marker element, followed by a zero, one or more
    /// non-marker elements, until another marker or the end of file is reached.
    ///
    /// Note that due to the fact that this reader exposes
    /// [lower level functions][Self::read_element], the marker element corresponding to an entry
    /// may have already been read. As such, this reader remembers the name of the last marker
    /// element read, and if the entry does not begin with an marker element, the remembered
    /// marker will be used instead.
    pub fn read_entry(&mut self) -> Result<Option<Entry>> {
        let first_element = self.read_element()?;
        if first_element.is_none() {
            return Ok(None);
        }

        let mut elements = Vec::with_capacity(1); // Entries usually have one element.
        let name = if let Some(Element::Marker(name)) = first_element {
            name
        } else {
            elements.push(first_element.unwrap());
            self.current_name.clone()
        };

        while let Some(element) = self.read_element()? {
            match element {
                Element::Marker(name) => {
                    self.retained_element = Some(Element::Marker(name));
                    break;
                }
                _ => {
                    elements.push(element);
                }
            }
        }

        Ok(Some(Entry { name, elements }))
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

        let name = self.read_utf8_string(size)?;
        self.current_name.clear();
        self.current_name.push_str(name.as_str());
        Ok(Element::Marker(name))
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
            0b0000 => self.read_utf8_string(size).map(Element::String),
            0b0100 => self.expect_heap(size).map(Element::Blob),
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
