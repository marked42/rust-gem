use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug)]
pub struct ChunkTypeError(String);

impl Display for ChunkTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid chunk type {}", self.0)
    }
}

impl From<String> for ChunkTypeError {
    fn from(value: String) -> Self {
        ChunkTypeError(value)
    }
}

impl From<&str> for ChunkTypeError {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}


#[derive(Debug)]
pub struct ChunkType {
    data: [u8; 4],
}

impl ChunkType {
    fn bytes(&self) -> [u8; 4] {
        self.data
    }

    fn is_valid_type_byte(val: u8) -> bool {
        val >= b'a' && val <= b'z' || val >= b'A' && val <= b'Z'
    }

    fn is_critical(&self) -> bool {
        (self.data[0] & 0b00100000) == 0
    }

    fn is_public(&self) -> bool {
        (self.data[1] & 0b00100000) == 0
    }

    fn is_reserved_bit_valid(&self) -> bool {
        (self.data[2] & 0b00100000) == 0
    }
    
    fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }

    fn is_safe_to_copy(&self) -> bool {
        (self.data[3] & 0b00100000) == 0b00100000
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = ChunkTypeError;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        for i in 0..4 {
            if !Self::is_valid_type_byte(value[i]) {
                // unicode 完全兼容 ascii，所以这里unsafe一定会成功
                let code = unsafe { str::from_utf8_unchecked(&value) };
                return Err(code.into());
            }
        }

        Ok(ChunkType { data: value })
    }
}

impl FromStr for ChunkType {
    type Err = ChunkTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != 4 {
            return Err(s.into());
        }

        let code: [u8; 4] =  bytes[0..4].try_into().unwrap();

        code.try_into()
    }
}

impl PartialEq for ChunkType {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { str::from_utf8_unchecked(&self.data) })
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
