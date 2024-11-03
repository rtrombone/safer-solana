//! Account serialization and deserialization utilities.

#[cfg(feature = "borsh")]
mod borsh;

use core::ops::{Deref, DerefMut};

#[cfg(feature = "alloc")]
use core::borrow::Borrow;

#[cfg(feature = "borsh")]
pub use borsh::*;

use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
};

use crate::discriminator::Discriminate;

/// Trait used to define a serializable account schema, which includes a discriminator. If the
/// account does not have a discriminator, use DISC_LEN == 0.
///
/// ### Examples
///
/// Implementing from scratch:
/// ```
/// use sealevel_tools::{account::AccountSerde, discriminator::{Discriminate, Discriminator}};
/// use solana_program::program_error::ProgramError;
///
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct Thing {
///     pub value: u64,
/// }
///
/// impl Discriminate<8> for Thing {
///     const DISCRIMINATOR: [u8; 8] = Discriminator::Sha2(b"state::Thing").to_bytes();
/// }
///
/// impl AccountSerde<8> for Thing {
///     fn try_deserialize_schema(data: &mut &[u8]) -> Result<Self, ProgramError> {
///         let encoded_value: [u8; 8] = data[..8].try_into().map_err(|_| ProgramError::InvalidAccountData)?;
///
///         Ok(Thing {
///             value: u64::from_le_bytes(encoded_value),
///         })
///     }
///
///    fn try_serialize_schema(&self, buf: &mut [u8]) -> Result<(), ProgramError> {
///         buf[..8].copy_from_slice(&self.value.to_le_bytes());
///         Ok(())
///     }
///
///     fn try_account_schema_space(&self) -> Result<u64, ProgramError> {
///         Ok(8)
///     }
/// }
/// ```
///
/// Using borsh:
/// ```
/// use borsh::{BorshDeserialize, BorshSerialize};
/// use sealevel_tools::{account::AccountSerde, discriminator::{Discriminate, Discriminator}};
///
/// #[derive(Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
/// pub struct Thing {
///     pub value: u64,
/// }
///
/// impl Discriminate<8> for Thing {
///     const DISCRIMINATOR: [u8; 8] = Discriminator::Sha2(b"state::Thing").to_bytes();
/// }
/// ```
pub trait AccountSerde<const DISC_LEN: usize>: Sized + Discriminate<DISC_LEN> {
    /// Deserialize the data from the given mutable slice of bytes.
    fn try_deserialize_schema(data: &mut &[u8]) -> Result<Self, ProgramError>;

    fn try_serialize_schema(&self, buf: &mut [u8]) -> Result<(), ProgramError>;

    /// Compute serialized length including its discriminator.
    fn try_account_schema_space(&self) -> Result<u64, ProgramError>;

    fn try_deserialize_data(data: &mut &[u8]) -> Result<Self, ProgramError> {
        let _: [u8; DISC_LEN] = match data[..DISC_LEN].try_into() {
            Ok(discriminator) if discriminator == Self::DISCRIMINATOR => discriminator,
            _ => {
                msg!("Invalid account discriminator");
                return Err(ProgramError::InvalidAccountData);
            }
        };

        Self::try_deserialize_schema(&mut &data[DISC_LEN..])
    }

    fn try_serialize_data(&self, mut buf: &mut [u8]) -> Result<(), ProgramError> {
        buf[..DISC_LEN].copy_from_slice(&Self::DISCRIMINATOR);

        buf = &mut buf[DISC_LEN..];
        self.try_serialize_schema(buf)
    }

    fn try_account_space(&self) -> Result<u64, ProgramError> {
        self.try_account_schema_space()
            .map(|len| len.saturating_add(DISC_LEN as u64))
    }
}

#[cfg(feature = "alloc")]
impl<const DISC_LEN: usize, T, U> AccountSerde<DISC_LEN> for alloc::boxed::Box<T>
where
    U: Into<alloc::boxed::Box<T>> + Borrow<T>,
    T: AccountSerde<DISC_LEN> + alloc::borrow::ToOwned<Owned = U> + ?Sized,
    T::Owned: AccountSerde<DISC_LEN>,
{
    #[inline(always)]
    fn try_deserialize_schema(data: &mut &[u8]) -> Result<Self, ProgramError> {
        T::Owned::try_deserialize_schema(data).map(Into::into)
    }

    fn try_serialize_schema(&self, buf: &mut [u8]) -> Result<(), ProgramError> {
        self.as_ref().try_serialize_schema(buf)
    }

    /// Compute serialized length including its discriminator.
    #[inline(always)]
    fn try_account_schema_space(&self) -> Result<u64, ProgramError> {
        self.as_ref().try_account_schema_space()
    }
}

/// Wrapper around a type implementing [Pack] and [IsInitialized].
pub struct PackAccountSchema<T: Pack + IsInitialized>(pub T);

impl<T: Pack + IsInitialized> Discriminate<0> for PackAccountSchema<T> {
    const DISCRIMINATOR: [u8; 0] = [];
}

impl<T: Pack + IsInitialized> Deref for PackAccountSchema<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Pack + IsInitialized> DerefMut for PackAccountSchema<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Pack + IsInitialized> AccountSerde<0> for PackAccountSchema<T> {
    /// Deserialize the data from the given mutable slice of bytes.
    #[inline(always)]
    fn try_deserialize_schema(data: &mut &[u8]) -> Result<Self, ProgramError> {
        T::unpack(data).map(Self)
    }

    fn try_serialize_schema(&self, buf: &mut [u8]) -> Result<(), ProgramError> {
        self.0.pack_into_slice(buf);

        Ok(())
    }

    /// Compute serialized length including its discriminator.
    #[inline(always)]
    fn try_account_schema_space(&self) -> Result<u64, ProgramError> {
        Ok(T::LEN as u64)
    }
}
