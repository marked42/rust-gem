use std::fmt::{Display, Formatter};
use std::ops::Range;

use crc::{CRC_32_ISO_HDLC, Crc};
use thiserror::Error;

use crate::Result;
use crate::chunk_type::ChunkType;

const PNG_CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

#[derive(Debug, Error)]
pub enum ChunkError {
    #[error("Too few bytes: received {0} bytes, expected as least {min} bytes)", min = ChunkLayout::MIN_SIZE)]
    TooShort(usize),
    #[error("Data too large: received {0} bytes, expected as most {max} bytes)", max = u32::MAX)]
    DataTooLarge(usize),
    #[error("Invalid chunk type: {0}")]
    InvalidChunkType(String),
    #[error("Invalid length: expected {expected}, got {actual}")]
    LengthMismatch { expected: usize, actual: usize },
    #[error("CRC mismatch: expected {expected}, got {actual}")]
    CrcMismatch { expected: u32, actual: u32 },
}

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
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> std::result::Result<Self, ChunkError> {
        if data.len() > u32::MAX as usize {
            return Err(ChunkError::DataTooLarge(data.len()).into());
        }

        Ok(Self::new_unchecked(chunk_type, data))
    }

    /// must be private to prevent from constructing invalid Chunk
    fn new_unchecked(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let crc = Self::calculate_crc(&chunk_type.bytes(), &data);

        Chunk {
            length: data.len() as u32,
            chunk_type,
            data,
            crc,
        }
    }

    pub fn next_chunk_length(value: &[u8]) -> Option<usize> {
        if value.len() < ChunkLayout::MIN_SIZE {
            return None;
        }

        // this is guaranteed success due to previous MIN_SIZE check
        let length_bytes: [u8; 4] = unsafe { ChunkLayout::length_bytes(value).unwrap_unchecked() };
        let data_length = u32::from_be_bytes(length_bytes) as usize;
        let total_length = ChunkLayout::total_length(data_length);

        if value.len() < total_length {
            return None;
        }

        Some(total_length)
    }

    /// can extract a chunk from bytes longer than required length, leaving some bytes unused
    pub fn from_bytes(value: &[u8]) -> std::result::Result<Self, ChunkError> {
        if value.len() < ChunkLayout::MIN_SIZE {
            return Err(ChunkError::TooShort(value.len()).into());
        }

        // this is guaranteed success due to previous MIN_SIZE check
        let length_bytes: [u8; 4] = unsafe { ChunkLayout::length_bytes(value).unwrap_unchecked() };

        let data_length = u32::from_be_bytes(length_bytes) as usize;
        let total_length = ChunkLayout::total_length(data_length);
        if value.len() < total_length {
            return Err(ChunkError::LengthMismatch {
                expected: total_length,
                actual: value.len(),
            }
            .into());
        }

        // this is guaranteed success due to previous MIN_SIZE check
        let type_bytes = unsafe { ChunkLayout::type_bytes(value).unwrap_unchecked() };
        let chunk_type: ChunkType = type_bytes
            .try_into()
            .map_err(|e| ChunkError::InvalidChunkType(format!("{:?}", e)))?;

        let data: Vec<u8> = ChunkLayout::data_bytes(value, data_length).to_vec();

        // this is guaranteed success due to previous MIN_SIZE check
        let crc_bytes = unsafe { ChunkLayout::crc_bytes(value, data_length).unwrap_unchecked() };
        let crc = u32::from_be_bytes(crc_bytes);
        let expected_crc = Self::calculate_crc(&chunk_type.bytes(), &data);
        if crc != expected_crc {
            return Err(ChunkError::CrcMismatch {
                expected: expected_crc,
                actual: crc,
            }
            .into());
        }

        Ok(Chunk {
            // data_length is valid u32
            length: data_length as u32,
            chunk_type,
            data,
            crc,
        })
    }

    pub fn calculate_crc(chunk_type: &[u8; 4], data: &[u8]) -> u32 {
        let mut digest = PNG_CRC.digest();
        digest.update(chunk_type);
        digest.update(data);
        digest.finalize()
    }

    pub fn verify_crc(chunk_type: &[u8; 4], data: &[u8], crc: u32) -> bool {
        let expected_crc = Self::calculate_crc(chunk_type, data);
        crc == expected_crc
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_as_string(&self) -> Result<String> {
        String::from_utf8(self.data.clone())
            .map_err(|e| format!("Invalid UTF-8 in chunk data: {}", e).into())
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn as_bytes(&self) -> Vec<u8> {
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
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, ChunkError> {
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
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data).unwrap();
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
