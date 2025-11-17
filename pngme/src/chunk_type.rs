use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::{Error, Result};

pub const BYTES_BIT_MASK: u8 = 0b00100000;
pub const BYTES_LENGTH: usize = 4;
pub type Bytes = [u8; BYTES_LENGTH];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkType {
    bytes: Bytes,
}

impl ChunkType {
    pub fn bytes(&self) -> Bytes {
        self.bytes
    }

    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.bytes).unwrap()
    }

    pub fn is_critical(&self) -> bool {
        (self.bytes[0] & BYTES_BIT_MASK) == 0
    }

    pub fn is_public(&self) -> bool {
        (self.bytes[1] & BYTES_BIT_MASK) == 0
    }

    pub fn is_reserved_bit_valid(&self) -> bool {
        (self.bytes[2] & BYTES_BIT_MASK) == 0
    }

    pub fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }

    /// bit 5 equals 1 means safe to copy
    pub fn is_safe_to_copy(&self) -> bool {
        (self.bytes[3] & BYTES_BIT_MASK) == BYTES_BIT_MASK
    }

    fn validate_bytes_value(value: &[u8]) -> Result<()> {
        if !value.iter().all(|c| c.is_ascii_alphabetic()) {
            return Err("Chunk type must contain only ASCII letters"
                .to_ascii_lowercase()
                .into());
        }
        Ok(())
    }

    fn validate_bytes_length(value: &[u8]) -> Result<()> {
        if value.len() != BYTES_LENGTH {
            return Err(
                format!("Chunk type must be exactly {} characters", BYTES_LENGTH)
                    .to_string()
                    .into(),
            );
        }

        Ok(())
    }

    pub fn validate_bytes(value: &[u8]) -> Result<String> {
        Self::validate_bytes_length(value)?;
        Self::validate_bytes_value(value)?;

        // unicode 完全兼容 ascii，所以这里unsafe一定会成功
        Ok(unsafe { String::from_utf8_unchecked(value.to_vec()) })
    }

    pub fn validate_str(value: &str) -> Result<String> {
        Self::validate_bytes(value.as_bytes())
    }
}

impl TryFrom<Bytes> for ChunkType {
    type Error = Error;

    fn try_from(value: Bytes) -> Result<Self> {
        Self::validate_bytes_value(&value)?;

        Ok(ChunkType { bytes: value })
    }
}

impl TryFrom<&[u8]> for ChunkType {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        Self::validate_bytes_length(value)?;
        let bytes: Bytes = value.try_into()?;
        bytes.try_into()
    }
}

impl FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.as_bytes().try_into()
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { str::from_utf8_unchecked(&self.bytes) })
    }
}

#[allow(unused_variables)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
