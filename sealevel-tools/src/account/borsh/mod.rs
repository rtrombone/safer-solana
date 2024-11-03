//! Borsh account serialization and deserialization utilities.

mod write;

use core::ops::{Deref, DerefMut};

use solana_program::program_error::ProgramError;
pub use write::*;

use borsh::{
    io::{Error, ErrorKind, Read, Result as IoResult, Write},
    BorshDeserialize, BorshSerialize,
};

use crate::{account::AccountSerde, discriminator::Discriminate};

/// This method first reads the expected discriminator from the reader and then deserializes the
/// data into the given type.
///
/// NOTE: This differs from borsh's `try_from_reader`, where this method does not check that all
/// bytes were consumed. If you need to perform this check, you should do so after calling this
/// method.
#[inline(always)]
pub fn try_read_borsh_data<const DISC_LEN: usize, T: BorshDeserialize>(
    reader: &mut impl Read,
    discriminator: Option<&[u8; DISC_LEN]>,
) -> IoResult<T> {
    match discriminator {
        None => T::deserialize_reader(reader),
        Some(discriminator)
            if DISC_LEN == 0 || &<[u8; DISC_LEN]>::deserialize_reader(reader)? == discriminator =>
        {
            T::deserialize_reader(reader)
        }
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid discriminator")),
    }
}

/// This method implements the same functionality as `try_read_data`, but instead of reading from a
/// reader, it reads from a mutable slice of bytes.
#[inline(always)]
pub fn try_deserialize_borsh_data<const DISC_LEN: usize, T: BorshDeserialize>(
    data: &mut &[u8],
    discriminator: Option<&[u8; DISC_LEN]>,
) -> IoResult<T> {
    try_read_borsh_data::<DISC_LEN, T>(data, discriminator)
}

/// This method first writes the discriminator to the writer and then serializes the data.
#[inline(always)]
pub fn try_write_borsh_data<const DISC_LEN: usize>(
    account_data: &impl BorshSerialize,
    writer: &mut impl Write,
    discriminator: Option<&[u8; DISC_LEN]>,
) -> IoResult<()> {
    if let Some(discriminator) = discriminator {
        writer.write_all(discriminator)?;
    }
    account_data.serialize(writer)
}

/// Wrapper around a type implementing [BorshDeserialize] and [BorshSerialize] with an assumed
/// discriminator (via [Discriminate]). If there is no discriminator, use DISC_LEN == 0.
pub struct BorshAccountSchema<
    const DISC_LEN: usize,
    T: Discriminate<DISC_LEN> + BorshDeserialize + BorshSerialize,
>(pub T);

impl<const DISC_LEN: usize, T: Discriminate<DISC_LEN> + BorshDeserialize + BorshSerialize>
    Discriminate<DISC_LEN> for BorshAccountSchema<DISC_LEN, T>
{
    const DISCRIMINATOR: [u8; DISC_LEN] = T::DISCRIMINATOR;
}

impl<const DISC_LEN: usize, T: Discriminate<DISC_LEN> + BorshDeserialize + BorshSerialize> Deref
    for BorshAccountSchema<DISC_LEN, T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const DISC_LEN: usize, T: Discriminate<DISC_LEN> + BorshDeserialize + BorshSerialize> DerefMut
    for BorshAccountSchema<DISC_LEN, T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const DISC_LEN: usize, T> AccountSerde<DISC_LEN> for BorshAccountSchema<DISC_LEN, T>
where
    T: Discriminate<DISC_LEN> + BorshDeserialize + BorshSerialize,
{
    /// Deserialize the data from the given mutable slice of bytes.
    #[inline(always)]
    fn try_deserialize_schema(data: &mut &[u8]) -> Result<Self, ProgramError> {
        T::deserialize(data).map(Self).map_err(Into::into)
    }

    fn try_serialize_schema(&self, mut buf: &mut [u8]) -> Result<(), ProgramError> {
        self.0.serialize(&mut buf).map_err(Into::into)
    }

    /// Compute serialized length including its discriminator.
    #[inline(always)]
    fn try_account_schema_space(&self) -> Result<u64, ProgramError> {
        borsh::object_length(&self.0)
            .map(|len| len as u64)
            .map_err(Into::into)
    }
}

impl<const DISC_LEN: usize, T> From<T> for BorshAccountSchema<DISC_LEN, T>
where
    T: Discriminate<DISC_LEN> + BorshDeserialize + BorshSerialize,
{
    fn from(data: T) -> Self {
        Self(data)
    }
}

#[cfg(feature = "alloc")]
#[cfg(test)]
mod test {
    use alloc::vec;

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

        let mut cursor = &data[..];
        assert_eq!(
            try_read_borsh_data::<4, ThingOne>(&mut cursor, Some(&disc)).unwrap(),
            ThingOne {
                a: [8, 9, 10],
                b: 42
            }
        );

        let mut remaining = vec![];
        assert_eq!(cursor.read_to_end(&mut remaining).unwrap(), 1);
        assert_eq!(remaining, vec![69]);

        #[derive(Debug, PartialEq, BorshDeserialize)]
        struct ThingTwo {
            a: [u8; 7],
            b: u32,
            c: u8,
        }

        let mut cursor = &data[..];
        assert_eq!(
            try_read_borsh_data::<4, ThingTwo>(&mut cursor, Some(&disc)).unwrap(),
            ThingTwo {
                a: [8, 9, 10, 42, 0, 0, 0],
                b: 0,
                c: 69,
            }
        );

        let mut remaining = vec![];
        assert_eq!(cursor.read_to_end(&mut remaining).unwrap(), 0);
        assert!(remaining.is_empty());
    }
}
