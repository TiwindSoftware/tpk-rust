use std::{io, mem};

use crate::Element;

/// Representation of a TPK write error.
#[derive(Debug)]
pub enum Error {
    /// A I/O error happened.
    IO(std::io::Error),
    /// An unknown error happened.
    ///
    /// This error is "technical unknown", it should only be used in cases where the user is not
    /// supposed to get an error but gets one anyway. More simply put, this error being returned
    /// anywhere should be considered a bug or a feature that is not yet implemented.
    Unknown,
}

/// Representation of a TPK write result.
pub type Result<T> = std::result::Result<T, Error>;

impl Element {
    /// Write the element to a given [writer][io::Write].
    ///
    /// This function will write the binary representation of the TPK element, including the type
    /// byte, size bytes and data bytes (if any).
    pub fn write(&self, writer: &mut dyn io::Write) -> Result<()> {
        write(writer, &[self.get_type_byte()])?;

        match *self {
            Element::Marker(ref val) => {
                let size = val.len();
                if size > 63 {
                    let remaining_size = size >> 6;
                    let dynsize = dyn_size(remaining_size);
                    write(writer, dynsize.as_slice())?;
                }
                write(writer, val.as_bytes())
            }
            Element::Folder => Ok(()),
            Element::Collection => Ok(()),
            Element::Integer8(val) => write(writer, &[val as u8]),
            Element::Integer16(val) => write(writer, &val.to_le_bytes()),
            Element::Integer32(val) => write(writer, &val.to_le_bytes()),
            Element::Integer64(val) => write(writer, &val.to_le_bytes()),
            Element::UInteger8(val) => write(writer, &[val]),
            Element::UInteger16(val) => write(writer, &val.to_le_bytes()),
            Element::UInteger32(val) => write(writer, &val.to_le_bytes()),
            Element::UInteger64(val) => write(writer, &val.to_le_bytes()),
            Element::Float32(val) => write(writer, &val.to_le_bytes()),
            Element::Float64(val) => write(writer, &val.to_le_bytes()),
            Element::Boolean(_) => Ok(()),
            Element::String(ref val) => {
                let bytes = val.as_bytes();
                write(writer, &static_size(bytes.len()))?;
                write(writer, bytes)
            }
            Element::Blob(ref val) => {
                write(writer, &static_size(val.len()))?;
                write(writer, val.as_slice())
            }
        }
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

#[inline(always)]
fn write(writer: &mut dyn io::Write, bytes: &[u8]) -> Result<()> {
    writer.write(bytes).map(|_| ()).map_err(Error::IO)
}
