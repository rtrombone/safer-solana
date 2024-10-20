use std::io::{Error, ErrorKind, Read, Result, Write};

use borsh::{BorshDeserialize, BorshSerialize};

/// This method first reads the expected discriminator from the reader and then deserializes the
/// data into the given type.
///
/// NOTE: This differs from borsh's `try_from_reader`, where this method does not check that all
/// bytes were consumed. If you need to perform this check, you should do so after calling this
/// method.
pub fn try_read_borsh_data<const N: usize, T: BorshDeserialize>(
    reader: &mut impl Read,
    discriminator: Option<&[u8; N]>,
) -> Result<T> {
    match discriminator {
        None => T::deserialize_reader(reader),
        Some(discriminator) if &<[u8; N]>::deserialize_reader(reader)? == discriminator => {
            T::deserialize_reader(reader)
        }
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid discriminator")),
    }
}

/// This method implements the same functionality as `try_read_data`, but instead of reading from a
/// reader, it reads from a mutable slice of bytes.
pub fn try_deserialize_borsh_data<const N: usize, T: BorshDeserialize>(
    data: &mut &[u8],
    discriminator: Option<&[u8; N]>,
) -> Result<T> {
    try_read_borsh_data::<N, T>(data, discriminator)
}

/// This method first writes the discriminator to the writer and then serializes the data.
pub fn try_write_borsh_data<const N: usize>(
    account_data: &impl BorshSerialize,
    writer: &mut impl Write,
    discriminator: Option<&[u8; N]>,
) -> Result<()> {
    if let Some(discriminator) = discriminator {
        writer.write_all(discriminator)?;
    }
    account_data.serialize(writer)
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::discriminator::Discriminator;

    use super::*;

    #[test]
    fn test_try_read_borsh_data() {
        let data = vec![229, 125, 11, 200, 8, 9, 10, 42, 0, 0, 0, 0, 0, 0, 0, 69];

        let disc = Discriminator::<4>::Keccak(b"a thing").to_bytes();
        assert_eq!(disc, <[u8; 4]>::try_from(&data[..4]).unwrap());

        #[derive(Debug, PartialEq, BorshDeserialize)]
        struct ThingOne {
            a: [u8; 3],
            b: u64,
        }

        let mut cursor = Cursor::new(data.clone());
        assert_eq!(
            try_read_borsh_data::<4, ThingOne>(&mut cursor, Some(&disc)).unwrap(),
            ThingOne {
                a: [8, 9, 10],
                b: 42
            }
        );

        let mut remaining = Vec::new();
        assert_eq!(cursor.read_to_end(&mut remaining).unwrap(), 1);
        assert_eq!(remaining, vec![69]);

        #[derive(Debug, PartialEq, BorshDeserialize)]
        struct ThingTwo {
            a: [u8; 7],
            b: u32,
            c: u8,
        }

        let mut cursor = Cursor::new(data);
        assert_eq!(
            try_read_borsh_data::<4, ThingTwo>(&mut cursor, Some(&disc)).unwrap(),
            ThingTwo {
                a: [8, 9, 10, 42, 0, 0, 0],
                b: 0,
                c: 69,
            }
        );

        let mut remaining = Vec::new();
        assert_eq!(cursor.read_to_end(&mut remaining).unwrap(), 0);
        assert!(remaining.is_empty());
    }
}
