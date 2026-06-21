//! The LC-3 `.obj` object-file format.
//!
//! An object file is a stream of 16-bit big-endian words: the first word is the
//! origin (the load address of the program), and each subsequent word is loaded
//! into consecutive memory locations starting there.

use std::error::Error;
use std::fmt;

/// An assembled LC-3 image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectFile {
    /// The address at which the first word is loaded.
    pub origin: u16,
    /// The program words, loaded consecutively beginning at [`origin`](Self::origin).
    pub words: Vec<u16>,
}

/// The reason a byte stream could not be decoded as a `.obj` image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObjectError {
    /// The stream was shorter than the single word required for the origin.
    MissingOrigin,
    /// The stream had an odd number of bytes; `.obj` words are 16 bits wide.
    TruncatedWord,
}

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOrigin => f.write_str("object file is missing its origin word"),
            Self::TruncatedWord => f.write_str("object file ends mid-word (odd byte length)"),
        }
    }
}

impl Error for ObjectError {}

impl ObjectFile {
    /// Decodes a big-endian `.obj` byte stream.
    ///
    /// The first word becomes the [`origin`](Self::origin); the rest become the
    /// program [`words`](Self::words). Returns [`ObjectError::MissingOrigin`] if
    /// there is no origin word and [`ObjectError::TruncatedWord`] if the stream
    /// does not divide into whole 16-bit words.
    pub fn from_be_bytes(bytes: &[u8]) -> Result<Self, ObjectError> {
        if bytes.len() < 2 {
            return Err(ObjectError::MissingOrigin);
        }
        if !bytes.len().is_multiple_of(2) {
            return Err(ObjectError::TruncatedWord);
        }

        let mut words = bytes
            .chunks_exact(2)
            .map(|word| u16::from_be_bytes([word[0], word[1]]));
        let origin = words.next().unwrap_or_default();

        Ok(Self {
            origin,
            words: words.collect(),
        })
    }

    /// Encodes the image as a big-endian `.obj` byte stream: the origin word
    /// followed by each program word.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        std::iter::once(self.origin)
            .chain(self.words.iter().copied())
            .flat_map(u16::to_be_bytes)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{ObjectError, ObjectFile};

    #[test]
    fn decodes_origin_and_words_big_endian() {
        let decoded = ObjectFile::from_be_bytes(&[0x30, 0x00, 0x12, 0x34, 0xAB, 0xCD])
            .expect("well-formed image");
        assert_eq!(decoded.origin, 0x3000);
        assert_eq!(decoded.words, vec![0x1234, 0xABCD]);
    }

    #[test]
    fn encode_decode_round_trips() {
        let image = ObjectFile {
            origin: 0x3000,
            words: vec![0xF026, 0x0000, 0xFFFF],
        };
        assert_eq!(
            ObjectFile::from_be_bytes(&image.to_be_bytes()),
            Ok(image.clone())
        );
    }

    #[test]
    fn empty_stream_has_no_origin() {
        assert_eq!(
            ObjectFile::from_be_bytes(&[]),
            Err(ObjectError::MissingOrigin)
        );
        assert_eq!(
            ObjectFile::from_be_bytes(&[0x30]),
            Err(ObjectError::MissingOrigin)
        );
    }

    #[test]
    fn odd_length_stream_is_truncated() {
        assert_eq!(
            ObjectFile::from_be_bytes(&[0x30, 0x00, 0x12]),
            Err(ObjectError::TruncatedWord)
        );
    }
}
