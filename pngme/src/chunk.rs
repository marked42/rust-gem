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
    pub const HEADER_SIZE: usize = Self::LENGTH_SIZE + Self::TYPE_SIZE;
    pub const MIN_SIZE: usize = Self::HEADER_SIZE + Self::CRC_SIZE;

    fn length_range() -> Range<usize> {
        0..Self::LENGTH_SIZE
    }

    fn length_bytes(bytes: &[u8]) -> Result<[u8; Self::LENGTH_SIZE]> {
        let b: [u8; Self::LENGTH_SIZE] = bytes[Self::length_range()].try_into()?;
        Ok(b)
    }

    fn type_range() -> Range<usize> {
        Self::LENGTH_SIZE..Self::HEADER_SIZE
    }

    fn type_bytes(bytes: &[u8]) -> Result<[u8; Self::TYPE_SIZE]> {
        let b: [u8; Self::TYPE_SIZE] = bytes[Self::type_range()].try_into()?;
        Ok(b)
    }

    fn data_range(data_length: usize) -> Range<usize> {
        let start = Self::HEADER_SIZE;
        start..start + data_length
    }

    fn data_bytes(bytes: &[u8], data_length: usize) -> &[u8] {
        &bytes[Self::data_range(data_length)]
    }

    fn crc_range(data_length: usize) -> Range<usize> {
        let start = Self::data_range(data_length).end;
        start..start + Self::CRC_SIZE
    }

    fn crc_bytes(bytes: &[u8], data_length: usize) -> Result<[u8; Self::CRC_SIZE]> {
        let b: [u8; Self::CRC_SIZE] = bytes[Self::crc_range(data_length)].try_into()?;
        Ok(b)
    }

    fn total_length(data_length: usize) -> usize {
        data_length + Self::MIN_SIZE
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
        let crc = Self::calculate_crc(&chunk_type.bytes(), &data);

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

        let length_bytes =
            ChunkLayout::length_bytes(value).map_err(|_| "Failed to extract length")?;
        let data_length = u32::from_be_bytes(length_bytes);

        let expected_total_length = ChunkLayout::total_length(data_length as usize);
        if value.len() != expected_total_length {
            return Err(format!(
                "actual length {} does not match expected length {} ",
                value.len(),
                expected_total_length
            )
            .into());
        }

        let type_bytes = ChunkLayout::type_bytes(value).map_err(|_| "Failed to extract type")?;
        let chunk_type: ChunkType = type_bytes.try_into()?;

        let data: Vec<u8> = ChunkLayout::data_bytes(value, data_length as usize).to_vec();

        let crc_bytes = ChunkLayout::crc_bytes(value, data_length as usize)
            .map_err(|_| "Failed to extract crc")?;
        let crc = u32::from_be_bytes(crc_bytes);
        if !Self::verify_crc(&chunk_type.bytes(), &data, crc) {
            return Err("invalid CRC".into());
        }

        Ok(Chunk {
            length: data_length,
            chunk_type,
            data,
            crc,
        })
    }

    fn calculate_crc(chunk_type: &[u8; 4], data: &[u8]) -> u32 {
        let mut digest = PNG_CRC.digest();
        digest.update(chunk_type);
        digest.update(data);
        digest.finalize()
    }

    fn verify_crc(chunk_type: &[u8; 4], data: &[u8], crc: u32) -> bool {
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
        &self.data
    }

    fn data_as_string(&self) -> Result<String> {
        String::from_utf8(self.data.clone())
            .map_err(|e| format!("Invalid UTF-8 in chunk data: {}", e).into())
    }

    fn crc(&self) -> u32 {
        self.crc
    }

    fn as_bytes(&self) -> Vec<u8> {
        let capacity = ChunkLayout::total_length(self.length as usize);
        let mut bytes: Vec<u8> = Vec::with_capacity(capacity);

        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.extend_from_slice(&self.chunk_type.bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.crc.to_be_bytes());

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
        write!(
            f,
            "Chunk {{ type: {}, length: {}, crc: {} }}",
            self.chunk_type, self.length, self.crc
        )
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
