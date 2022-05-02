use crate::Element;
use std::{io, mem};
use thiserror::Error;

/// Representation of a TPK write error.
#[derive(Error, Debug)]
pub enum Error {
    /// An unknown error happened.
    ///
    /// This error is "technical unknown", it should only be used in cases where the user is not
    /// supposed to get an error but gets one anyway. More simply put, this error being returned
    /// anywhere should be considered a bug or a feature that is not yet implemented.
    #[error("Unknown error")]
    Unknown,

    /// A I/O error happened.
    #[error("I/O error while writing TPK data: {source}")]
    Io {
        #[from]
        source: io::Error,
    },
}

/// Representation of a TPK write result.
pub type Result<T> = std::result::Result<T, Error>;

/// A TPK writer structure.
///
/// This structure holds the destination to which TPK data should be written.
pub struct Writer<T> {
    write: T,
}

impl<T> Writer<T>
    where T: io::Write,
{
    /// Create a new [TPK writer][Writer].
    pub fn new(write: T) -> Writer<T> {
        Writer { write }
    }

    /// Write the given [Element] to this writer.
    ///
    /// This function will write the binary representation of the TPK element, including the type
    /// byte, size bytes and data bytes (if any).
    pub fn write_element(&mut self, element: &Element) -> Result<()> {
        self.write.write_all(&[element.get_type_byte()])?;

        match *element {
            Element::Marker(ref val) => {
                let size = val.len();
                if size > 63 {
                    let remaining_size = size >> 6;
                    let dynsize = dyn_size(remaining_size);
                    self.write.write_all(dynsize.as_slice())?;
                }
                self.write.write_all(val.as_bytes())?;
            }
            Element::Integer8(val) => {
                self.write.write_all(&[val as u8])?;
            }
            Element::Integer16(val) => {
                self.write.write_all(&val.to_le_bytes())?;
            }
            Element::Integer32(val) => {
                self.write.write_all(&val.to_le_bytes())?;
            }
            Element::Integer64(val) => {
                self.write.write_all(&val.to_le_bytes())?;
            }
            Element::UInteger8(val) => {
                self.write.write_all(&[val])?;
            }
            Element::UInteger16(val) => {
                self.write.write_all(&val.to_le_bytes())?;
            }
            Element::UInteger32(val) => {
                self.write.write_all(&val.to_le_bytes())?;
            }
            Element::UInteger64(val) => {
                self.write.write_all(&val.to_le_bytes())?;
            }
            Element::Float32(val) => {
                self.write.write_all(&val.to_le_bytes())?;
            }
            Element::Float64(val) => {
                self.write.write_all(&val.to_le_bytes())?;
            }
            Element::String(ref val) => {
                let bytes = val.as_bytes();
                self.write.write_all(&static_size(bytes.len()))?;
                self.write.write_all(bytes)?;
            }
            Element::Blob(ref val) => {
                self.write.write_all(&static_size(val.len()))?;
                self.write.write_all(val.as_slice())?;
            }
            _ => (),
        };
        Ok(())
    }
}

fn static_size(size: usize) -> Vec<u8> {
    match size {
        0..=255 => Vec::from([size as u8]),
        256..=65535 => Vec::from((size as u16).to_le_bytes()),
        65536..=4294967295 => Vec::from((size as u32).to_le_bytes()),
        _ => Vec::from((size as u64).to_le_bytes()),
    }
}

fn dyn_size(size: usize) -> Vec<u8> {
    if size == 0 {
        return Vec::from([0u8]);
    }

    let fls = if size == 0 {
        0u32
    } else {
        mem::size_of::<usize>() as u32 * 8u32 - size.leading_zeros()
    };

    let mut ret = Vec::with_capacity((fls / 7) as usize);

    let mut size = size;
    while size > 0 {
        ret.push((size as u8 & 0x7F) | if size > 0x7F { 0b10000000u8 } else { 0u8 });
        size >>= 7;
    }
    ret
}
