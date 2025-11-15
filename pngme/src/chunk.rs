use crate::chunk_type::ChunkType;
use crate::{Error, Result};
use crc::{CRC_32_ISO_HDLC, Crc};
use std::fmt::{Display, Formatter};
use std::ops::Range;

const PNG_CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

struct ChunkLayout;

impl ChunkLayout {
    pub const LENGTH_SIZE: usize = 4;
    pub const TYPE_SIZE: usize = 4;
    pub const CRC_SIZE: usize = 4;
    pub const MIN_SIZE: usize = Self::LENGTH_SIZE + Self::TYPE_SIZE + Self::CRC_SIZE;

    fn length_range() -> Range<usize> {
        0..Self::LENGTH_SIZE
    }

    fn length_bytes(bytes: &[u8]) -> Result<[u8; Self::LENGTH_SIZE]> {
        let b: [u8; Self::LENGTH_SIZE] = bytes[Self::length_range()].try_into()?;
        Ok(b)
    }

    fn type_range() -> Range<usize> {
        Self::length_range().end..(Self::length_range().end + Self::TYPE_SIZE)
    }

    fn type_bytes(bytes: &[u8]) -> Result<[u8; Self::TYPE_SIZE]> {
        let b: [u8; Self::TYPE_SIZE] = bytes[Self::type_range()].try_into()?;
        Ok(b)
    }

    fn data_range(data_length: usize) -> Range<usize> {
        Self::type_range().end..(Self::type_range().end + data_length)
    }

    fn data_bytes(bytes: &[u8], data_length: usize) -> &[u8] {
        &bytes[Self::data_range(data_length)]
    }

    fn crc_range(data_length: usize) -> Range<usize> {
        Self::data_range(data_length).end..(Self::data_range(data_length).end + Self::CRC_SIZE)
    }

    fn crc_bytes(bytes: &[u8], data_length: usize) -> Result<[u8; Self::CRC_SIZE]> {
        let b: [u8; Self::CRC_SIZE] = bytes[Self::crc_range(data_length)].try_into()?;
        Ok(b)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let crc = Self::calculate_crc(chunk_type.bytes(), data.as_slice());

        Chunk {
            length: data.len() as u32,
            chunk_type,
            data,
            crc,
        }
    }

    fn from_bytes(value: &[u8]) -> Result<Self> {
        if value.len() < ChunkLayout::MIN_SIZE {
            return Err(format!(
                "bytes length {} less than min chunk length {}",
                value.len(),
                ChunkLayout::MIN_SIZE
            )
            .into());
        }

        let data_length = u32::from_be_bytes(ChunkLayout::length_bytes(value)?);

        let actual_data_length = value.len() as u32 - 12;
        if data_length != actual_data_length {
            return Err(format!(
                "actual data length {} does not match specified data length {} ",
                actual_data_length, data_length
            )
            .into());
        }

        let chunk_type: ChunkType = ChunkLayout::type_bytes(value)?.try_into()?;

        let data: Vec<u8> = ChunkLayout::data_bytes(value, data_length as usize).try_into()?;

        let crc = u32::from_be_bytes(ChunkLayout::crc_bytes(value, data_length as usize)?);
        if !Self::verify_crc(chunk_type.bytes(), data.as_slice(), crc) {
            return Err("invalid CRC".into());
        }

        Ok(Chunk {
            length: data_length,
            chunk_type,
            data,
            crc,
        })
    }

    fn calculate_crc(chunk_type: [u8; 4], data: &[u8]) -> u32 {
        let mut digest = PNG_CRC.digest();
        digest.update(&chunk_type);
        digest.update(data);
        digest.finalize()
    }

    fn verify_crc(chunk_type: [u8; 4], data: &[u8], crc: u32) -> bool {
        let expected_crc = Self::calculate_crc(chunk_type, data);
        crc == expected_crc
    }

    fn length(&self) -> u32 {
        self.length
    }

    fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn data_as_string(&self) -> Result<String> {
        match String::from_utf8(self.data.clone()) {
            Ok(s) => Ok(s),
            Err(e) => Err(e.to_string().into()),
        }
    }

    fn crc(&self) -> u32 {
        self.crc
    }

    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(self.length.to_be_bytes().as_slice());
        bytes.extend_from_slice(self.chunk_type.bytes().as_slice());
        bytes.extend_from_slice(self.data.as_slice());
        bytes.extend_from_slice(self.crc.to_be_bytes().as_slice());

        bytes
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        Self::from_bytes(value)
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "length: {}, type: {}", self.length, self.chunk_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
