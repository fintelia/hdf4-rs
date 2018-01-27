use std::error::Error;
use byteorder::{BigEndian, ByteOrder};

#[derive(Clone, Debug)]
pub struct TagRef {
    tag: u16,
    reference: u16,
}

#[derive(Clone, Debug)]
pub enum Tag {
    Null,
    Version {
        majorv: u32,
        minorv: u32,
        release: u32,
        string: String,
    },
    NumberType {
        version: u8,
        type_: u8,
        width: u8,
        class: u8,
    },
    FileIdentifier { character_string: String },
    ScientificDataDimension {
        dimensions: Vec<u32>,
        datatype: TagRef,
        scale: Vec<TagRef>,
    },
    /// Tag of unknown type.
    Unknown { tag: u16, data: Vec<u8> },
    /// Tag with invalid offset and length.
    Invalid(u16),
    /// A tag of recognized type that failed to parse.
    Corrupt(u16),
}

impl Tag {
    fn from_raw_inner(tag: u16, data: &[u8]) -> Result<Self, Box<Error>> {
        let read_u16 = |offset| BigEndian::read_u16(&data[offset..offset + 2]);
        let read_u32 = |offset| BigEndian::read_u32(&data[offset..offset + 4]);
        let read_tagref = |offset| {
            TagRef {
                tag: read_u16(offset),
                reference: read_u16(offset + 2),
            }
        };

        Ok(match tag {
            1 => Tag::Null,
            30 if data.len() < 12 => Tag::Corrupt(30),
            30 => Tag::Version {
                majorv: read_u32(0),
                minorv: read_u32(4),
                release: read_u32(8),
                string: String::from_utf8(data[12..].to_vec())?,
            },
            106 if data.len() != 4 => Tag::Corrupt(106),
            106 => Tag::NumberType {
                version: data[0],
                type_: data[1],
                width: data[2],
                class: data[3],
            },
            701 if data.len() < 2 => Tag::Corrupt(701),
            701 => {
                let rank = read_u16(0) as usize;
                if data.len() < 2 + rank * 4 + 4 + rank * 4 {
                    Tag::Corrupt(701)
                } else {
                    Tag::ScientificDataDimension {
                        dimensions: (0..rank).map(|r| read_u32(2 + r * 4)).collect(),
                        datatype: read_tagref(2 + rank * 4),
                        scale: (0..rank)
                            .map(|r| read_tagref(2 + rank * 4 + 4 + r * 4))
                            .collect(),
                    }
                }
            }
            _ => Tag::Unknown {
                tag,
                data: data.to_vec(),
            },
        })
    }

    pub(crate) fn from_raw(tag: u16, data: &[u8]) -> Self {
        Self::from_raw_inner(tag, data).unwrap_or(Tag::Corrupt(tag))
    }
}
