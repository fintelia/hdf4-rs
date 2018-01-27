extern crate byteorder;

use byteorder::{BigEndian, ByteOrder};

mod tag;
pub use tag::Tag;

#[derive(Clone, Debug)]
pub enum ParseError {
    IncompleteFile,
    InvalidMagicNumber,
}

#[derive(Clone, Debug)]
pub struct DataDescriptor {
    pub tag: Tag,
    pub reference: u16,
}

#[derive(Clone, Debug)]
pub struct HdfFile {
    pub descriptors: Vec<DataDescriptor>,
}
impl HdfFile {
    pub fn from_slice(slice: &[u8]) -> Result<Self, ParseError> {
        let read_u16 = |offset| BigEndian::read_u16(&slice[offset..offset + 2]);
        let read_u32 = |offset| BigEndian::read_u32(&slice[offset..offset + 4]);

        if slice.len() < 10 {
            return Err(ParseError::IncompleteFile);
        }
        if slice[0..4] != [0x0e, 0x03, 0x13, 0x01] {
            return Err(ParseError::InvalidMagicNumber);
        }

        let mut descriptors = Vec::new();

        let mut next_block: usize = 4;
        while next_block != 0 {
            let mut cursor = next_block + 6;
            let block_size = read_u16(next_block) as usize;
            next_block = read_u32(next_block + 2) as usize;

            if slice.len() < cursor + block_size * 12 {
                return Err(ParseError::IncompleteFile);
            }
            for _ in 0..block_size {
                let tag = read_u16(cursor);
                let reference = read_u16(cursor + 2);
                let offset = read_u32(cursor + 4) as usize;
                let length = read_u32(cursor + 8) as usize;

                let data = if offset as u64 + length as u64 > slice.len() as u64 {
                    &[]
                } else {
                    &slice[offset..(offset + length)]
                };

                descriptors.push(DataDescriptor {
                    tag: Tag::from_raw(tag, data),
                    reference,
                });
                cursor += 12;
            }
        }

        Ok(Self { descriptors })
    }

    pub fn remove_nulls(&mut self) {
        let not_null = |d: &DataDescriptor| if let Tag::Null = d.tag { false } else { true };
        self.descriptors.retain(not_null);
    }
}
