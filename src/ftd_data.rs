use std::collections::HashMap;

use base64::prelude::*;
use uuid::Uuid;

/// Ftd's uuid's have a different notation, so this converts a uuid parsed from ftd's notation to a regular uuid
pub fn ftd_uuid_to_uuid(uuid: Uuid) -> Uuid {
    let mut bytes = uuid.into_bytes();

    let mut swap = |a, b| {
        let tmp = bytes[a];
        bytes[a] = bytes[b];
        bytes[b] = tmp;
    };

    swap(0, 3);
    swap(1, 2);

    swap(4, 5);

    swap(6, 7);

    Uuid::from_bytes(bytes)
}


#[derive(Debug, Default)]
pub struct BlueprintData {
    data: HashMap<BlockIndex, BlockData>,
}

impl BlueprintData {
    pub fn add_block_data(&mut self, index: BlockIndex, data: BlockData) {
        self.data.insert(index, data);
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serializer = Serializer::default();

        for (block_index, block) in self.data.iter() {
            // FIXME: don't allocate for every block, just easier to do now to get length before inserting into main buffer
            let mut block_data_serializer = Serializer::default();
            let block_serialize_info = block.serialize_to(&mut block_data_serializer);

            serializer.push_u24(block_index.0);
            serializer.push_u16(block_serialize_info.header_len.try_into().unwrap());

            // this bytes are unused
            serializer.push_u16(0);

            // push body length, ftd has wierd way of storing data body length
            let mut body_length = block_serialize_info.data_length;
            loop {
                if body_length >= u16::MAX as usize {
                    serializer.push_u16(u16::MAX);
                    body_length -= u16::MAX as usize;
                } else {
                    serializer.push_u16(body_length.try_into().unwrap());
                    break;
                }
            }

            serializer.push_bytes(block_data_serializer.as_slice());
        }

        serializer.into_inner()
    }

    /// Creates a serialized base64 string which should be inserted in the block data section of an ftd blueprint file
    pub fn serialize_to_bp_data_string(&self) -> String {
        let data = self.serialize();
        BASE64_STANDARD.encode(&data)
    }
}

#[derive(Debug, Default)]
pub struct BlockData {
    sections: HashMap<SectionId, SectionData>,
}

struct BlockDataSerializeInfo {
    header_len: usize,
    data_length: usize,
}

impl BlockData {
    pub fn add_section_data(&mut self, id: SectionId, data: SectionData) {
        self.sections.insert(id, data);
    }

    fn serialize_to(&self, serializer: &mut Serializer) -> BlockDataSerializeInfo {
        // serialize data body seperate from headers so we know offsets from start of data easily
        let mut data_body = Serializer::default();

        for (section_id, section) in self.sections.iter() {
            let section_start_offset = data_body.len();
            section.serialize_to(&mut data_body);

            serializer.push_u24(section_id.0);
            serializer.push_wierd_u32(section_start_offset.try_into().unwrap());
        }

        serializer.push_bytes(data_body.as_slice());

        BlockDataSerializeInfo {
            header_len: 7 * self.sections.len(),
            data_length: data_body.len(),
        }
    }
}

#[derive(Debug, Default)]
pub struct SectionData {
    entries: HashMap<DataEntryId, DataEntry>,
}

impl SectionData {
    pub fn add_entry(&mut self, id: DataEntryId, entry: DataEntry) {
        self.entries.insert(id, entry);
    }

    pub fn with_entry(mut self, id: DataEntryId, entry: DataEntry) -> Self {
        self.entries.insert(id, entry);
        self
    }

    fn serialize_to(&self, serializer: &mut Serializer) {
        for (entry_id, entry_data) in self.entries.iter() {
            entry_data.serialize_to(*entry_id, serializer);
        }
    }
}

#[derive(Debug)]
pub enum DataEntry {
    Bool(bool),
    U32(u32),
    I32(i32),
    F32(f32),
    I64(i64),
    F64(f64),
    Vector2(Vector2),
    Uuid(Uuid),
    Bytes(Vec<u8>),
    String(String),
}

impl DataEntry {
    fn serialize_to(&self, entry_id: u16, serializer: &mut Serializer) {
        serializer.push_u16(entry_id);

        match self {
            Self::Bool(val) => {
                serializer.push_u8(1);
                serializer.push_u8(*val as u8);
            },
            Self::U32(val) => {
                serializer.push_u8(4);
                serializer.push_bytes(&val.to_le_bytes());
            },
            Self::I32(val) => {
                serializer.push_u8(4);
                serializer.push_bytes(&val.to_le_bytes());
            },
            Self::F32(val) => {
                serializer.push_u8(4);
                serializer.push_f32(*val);
            },
            Self::I64(val) => {
                serializer.push_u8(8);
                serializer.push_bytes(&val.to_le_bytes());
            },
            Self::F64(val) => {
                serializer.push_u8(8);
                serializer.push_bytes(&val.to_le_bytes());
            },
            Self::Vector2(val) => {
                serializer.push_u8(8);
                serializer.push_f32(val.x);
                serializer.push_f32(val.y);
            },
            Self::Uuid(val) => {
                serializer.push_u8(16);
                serializer.push_bytes(val.as_bytes());
            },
            Self::Bytes(val) => Self::serialize_bytes(val.as_slice(), entry_id, serializer),
            Self::String(val) => Self::serialize_bytes(val.as_bytes(), entry_id, serializer),
        }
    }

    /// Serializes bytes, used for string and bytes
    /// 
    /// When called caller should ensure first entry id is already present
    fn serialize_bytes(mut data: &[u8], entry_id: u16, serializer: &mut Serializer) {
        loop {
            let write_amount = std::cmp::min(data.len(), 255);

            serializer.push_u8(write_amount.try_into().unwrap());
            serializer.push_bytes(&data[..write_amount]);

            data = &data[write_amount..]; 
            if data.len() == 0 {
                break;
            }

            // more data to write, ftd represents this with another consecutive element with the same id
            serializer.push_u16(entry_id);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vector2 {
            x,
            y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockIndex(u32);

impl BlockIndex {
    pub fn new(n: u32) -> Self {
        if n >= 1 << 24 {
            panic!("block index to big to fit in 3 bytes");
        }

        BlockIndex(n)
    }
}

impl From<u32> for BlockIndex {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionId(u32);

impl SectionId {
    pub fn new(n: u32) -> Self {
        if n >= 1 << 24 {
            panic!("section id to big to fit in 3 bytes");
        }

        SectionId(n)
    }
}

impl From<u32> for SectionId {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

pub type DataEntryId = u16;

#[derive(Default)]
struct Serializer {
    data: Vec<u8>,
}

impl Serializer {
    fn push_u8(&mut self, n: u8) {
        self.data.push(n);
    }

    fn push_u16(&mut self, n: u16) {
        self.data.extend_from_slice(&n.to_le_bytes());
    }

    fn push_u24(&mut self, n: u32) {
        assert!(n < (1 << 24), "could not push value into serializer, it is larger than 3 bytes");
        self.data.extend_from_slice(&n.to_le_bytes()[..3]);
    }

    /// Some fields in ftd are 32 bit integers but the bytes are stored in a wierd order
    /// 
    /// This corresponde to LegacyInteger or something called like it in ftd code
    fn push_wierd_u32(&mut self, n: u32) {
        self.push_u16((n >> 16) as u16);
        self.push_u16(n as u16);
    }

    fn push_f32(&mut self, n: f32) {
        self.data.extend_from_slice(&n.to_le_bytes());
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn into_inner(self) -> Vec<u8> {
        self.data
    }
}