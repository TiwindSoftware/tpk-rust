/// Representation of a TPK element.
///
/// TPK elements are the building block of Tiwind Packages: they contain a single piece of data or
/// metadata but, when put together, can describe complex and structured data.
#[derive(Debug)]
pub enum Element {
    /// Represents a TPK marker.
    Marker(String),
    /// Represents a TPK folder.
    Folder,
    /// Represents a TPK collection.
    Collection,
    /// Represents a signed 8-bit TPK integer.
    Integer8(i8),
    /// Represents a signed 16-bit TPK integer.
    Integer16(i16),
    /// Represents a signed 32-bit TPK integer.
    Integer32(i32),
    /// Represents a signed 64-bit TPK integer.
    Integer64(i64),
    /// Represents a unsigned 8-bit TPK integer.
    UInteger8(u8),
    /// Represents a unsigned 16-bit TPK integer.
    UInteger16(u16),
    /// Represents a unsigned 32-bit TPK integer.
    UInteger32(u32),
    /// Represents a unsigned 64-bit TPK integer.
    UInteger64(u64),
    /// Represents a signed 32-bit TPK single precision floating-point number.
    Float32(f32),
    /// Represents a signed 64-bit TPK double precision floating-point number.
    Float64(f64),
    /// Represents a TPK boolean.
    Boolean(bool),
    /// Represents a TPK UTF-8 string.
    String(String),
    /// Represents a TPK binary blob.
    Blob(Vec<u8>),
}

/// Representation of a TPK entry.
///
/// A TPK entry is composed of a name and zero, one or more associated elements.
pub struct Entry {
    pub name: String,
    pub elements: Vec<Element>,
}

impl Element {
    /// Get the type byte for this [Element].
    pub fn get_type_byte(&self) -> u8 {
        match *self {
            Element::Marker(ref val) => {
                let size = val.len();
                0b10000000u8
                    | (if size > 63 { 0b01000000u8 } else { 0u8 })
                    | (size & 0b00111111) as u8
            }
            Element::Folder => 0u8,
            Element::Collection => 1u8,
            Element::Integer8(_) => 0b00100100u8,
            Element::Integer16(_) => 0b00100101u8,
            Element::Integer32(_) => 0b00100110u8,
            Element::Integer64(_) => 0b00100111u8,
            Element::UInteger8(_) => 0b00100000u8,
            Element::UInteger16(_) => 0b00100001u8,
            Element::UInteger32(_) => 0b00100010u8,
            Element::UInteger64(_) => 0b00100011u8,
            Element::Float32(_) => 0b00101110u8,
            Element::Float64(_) => 0b00101111u8,
            Element::Boolean(val) => {
                if val {
                    0b00110001u8
                } else {
                    0b00110000u8
                }
            }
            Element::String(ref val) => 0b00010000u8 | size_byte(val.len()),
            Element::Blob(ref val) => 0b00010100u8 | size_byte(val.len()),
        }
    }
}

#[inline(always)]
fn size_byte(size: usize) -> u8 {
    match size {
        0..=255 => 0b00u8,
        256..=65535 => 0b01u8,
        65536..=4294967295 => 0b10u8,
        _ => 0b11u8,
    }
}
