//! Discriminator generation for program accounts, events, and instructions.

use core::borrow::Borrow;

use alloc::{borrow::ToOwned, boxed::Box};

/// Discriminator generated either by user-defined or by specific hashing function (where total hash
/// output is 256 bits). These discriminators can be used for discriminating against serialized
/// program accounts, serialized events, and instructions (as selectors for specific program
/// instructions).
///
/// Only Keccak, Sha2 and Sha3 hashing are supported.
///
/// ### Example
///
/// ```
/// use sealevel_tools::discriminator::Discriminator;
///
/// const DISCRIMINATOR: [u8; 7] = Discriminator::Defined([1, 2, 3, 4, 5, 6, 7]).to_bytes();
/// const ANOTHER_DISCRIMINATOR: [u8; 4] = Discriminator::Keccak(b"another one").to_bytes();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Discriminator<'a, const LEN: usize> {
    Defined([u8; LEN]),

    // Using keccak hasher.
    Keccak(&'a [u8]),

    // Using sha2 hasher. Discrminators used in anchor-lang and spl-discriminator crates use the
    // first 8 bytes of this type.
    Sha2(&'a [u8]),

    /// Using sha3 hasher.
    Sha3(&'a [u8]),
}

impl<'a, const LEN: usize> Discriminator<'a, LEN> {
    /// Be careful when using this method because there is no const constraint that prevents LEN to be
    /// larger than a computed digest's length (32 bytes). If LEN is larger, the output will be padded
    /// to the right with zeros.
    pub const fn to_bytes(self) -> [u8; LEN] {
        let digest = match self {
            Discriminator::Defined(disc) => return disc,
            Discriminator::Keccak(input) => const_crypto::sha3::Keccak256::new()
                .update(input)
                .finalize(),
            Discriminator::Sha2(input) => {
                const_crypto::sha2::Sha256::new().update(input).finalize()
            }
            Discriminator::Sha3(input) => {
                const_crypto::sha3::Sha3_256::new().update(input).finalize()
            }
        };

        let mut inner = [0; LEN];
        let mut i = 0;
        loop {
            if i >= LEN || i >= digest.len() {
                break;
            }

            inner[i] = digest[i];
            i += 1;
        }

        inner
    }
}

/// Simple trait to enforce a discriminator for a type. This type is used for various account
/// handling in this crate (specifically serialization/deserialization). Defining
/// [Discriminate::DISCRIMINATOR] can be used in conjunction with [Discriminator] to generate a
/// unique discriminator for a type.
///
/// ### Example
///
/// ```
/// use sealevel_tools::discriminator::{Discriminate, Discriminator};
///
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct Thing {
///     data: u64
/// }
///
/// impl Discriminate<5> for Thing {
///    const DISCRIMINATOR: [u8; 5] = [1, 2, 3, 4, 5];
/// }
///
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct AnotherThing {
///    data: u64
/// }
///
/// impl Discriminate<4> for AnotherThing {
///   const DISCRIMINATOR: [u8; 4] = Discriminator::Sha3(b"AnotherThing").to_bytes();
/// }
/// ```
pub trait Discriminate<const LEN: usize> {
    /// Fixed-bytes discriminator. This can be used with [Discriminator] to generate bytes based on
    /// a hash.
    const DISCRIMINATOR: [u8; LEN];
}

impl<const LEN: usize, T, U> Discriminate<LEN> for Box<T>
where
    U: Into<Box<T>> + Borrow<T>,
    T: Discriminate<LEN> + ToOwned<Owned = U> + ?Sized,
    T::Owned: Discriminate<LEN>,
{
    const DISCRIMINATOR: [u8; LEN] = T::DISCRIMINATOR;
}

#[cfg(test)]
mod test {
    use super::*;

    const DEFINED_DISCRIMINATOR: [u8; 8] =
        Discriminator::Defined([1, 2, 3, 4, 5, 6, 7, 8]).to_bytes();
    const KECCAK_DISCRIMINATOR: [u8; 8] = Discriminator::Keccak(b"a thing").to_bytes();
    const SHA2_DISCRIMINATOR: [u8; 8] = Discriminator::Sha2(b"a thing").to_bytes();
    const SHA3_DISCRIMINATOR: [u8; 8] = Discriminator::Sha3(b"a thing").to_bytes();

    #[test]
    fn test_constants() {
        assert_eq!(DEFINED_DISCRIMINATOR, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(KECCAK_DISCRIMINATOR, [229, 125, 11, 200, 40, 184, 43, 53]);
        assert_eq!(SHA2_DISCRIMINATOR, [184, 38, 71, 117, 18, 58, 226, 106]);
        assert_eq!(SHA3_DISCRIMINATOR, [42, 209, 81, 98, 71, 17, 253, 121]);
    }

    #[test]
    fn test_defined() {
        let discriminator = Discriminator::Defined([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let bytes = discriminator.to_bytes();
        assert_eq!(bytes, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(&bytes[..8], &DEFINED_DISCRIMINATOR);
    }

    #[test]
    fn test_keccak() {
        let discriminator = Discriminator::Keccak(b"a thing");
        let bytes = discriminator.to_bytes();
        assert_eq!(bytes, [229, 125, 11, 200, 40, 184, 43, 53, 164]);
        assert_eq!(&bytes[..8], &KECCAK_DISCRIMINATOR);
    }

    #[test]
    fn test_sha2() {
        let discriminator = Discriminator::Sha2(b"a thing");
        let bytes = discriminator.to_bytes();
        assert_eq!(bytes, [184, 38, 71, 117, 18, 58, 226, 106, 248]);
        assert_eq!(&bytes[..8], &SHA2_DISCRIMINATOR);
    }

    #[test]
    fn test_sha3() {
        let discriminator = Discriminator::Sha3(b"a thing");
        let bytes = discriminator.to_bytes();
        assert_eq!(bytes, [42, 209, 81, 98, 71, 17, 253, 121, 134]);
        assert_eq!(&bytes[..8], &SHA3_DISCRIMINATOR);
    }

    #[test]
    fn test_spl_discriminator_equivalence() {
        use spl_discriminator::SplDiscriminate;

        #[derive(SplDiscriminate)]
        #[discriminator_hash_input("a thing")]
        struct Thing;

        assert_eq!(Thing::SPL_DISCRIMINATOR.as_slice(), &SHA2_DISCRIMINATOR);
    }

    #[test]
    #[ignore]
    fn test_anchor_account_discriminator_equivalence() {
        // use anchor_lang::{prelude::*, Discriminator as AnchorDiscriminator};

        // #[event]
        // struct Thing {
        //     a: u64,
        // }

        // assert_eq!(
        //     Discriminator::<8>::Sha2(b"event:Thing").to_bytes(),
        //     Thing::DISCRIMINATOR
        // );
        todo!("Re-introduce when anchor-lang supports 2.0");
    }

    #[test]
    #[ignore]
    fn test_anchor_event_cpi_selector_equivalence() {
        // // NOTE: The EVENT_IX_TAG_LE is backwards in anchor-lang... the hex representation of the
        // // u64 value represents the first 8 bytes of the sha2 hash of "anchor:event" but the hex is
        // // supposed to be read in as big-endian in order to preserve this order.
        // //
        // // It is unclear whether the anchor-lang contributor intended for the selector to be
        // // backwards.
        // const EXPECTED: [u8; 8] = [0x1d, 0x9a, 0xcb, 0x51, 0x2e, 0xa5, 0x45, 0xe4];

        // assert_eq!(
        //     Discriminator::<8>::Sha2(b"anchor:event").to_bytes(),
        //     EXPECTED
        // );
        // assert_eq!(
        //     anchor_lang::event::EVENT_IX_TAG,
        //     u64::from_be_bytes(EXPECTED)
        // );
        // assert_ne!(anchor_lang::event::EVENT_IX_TAG_LE, EXPECTED);
        todo!("Re-introduce when anchor-lang supports 2.0");
    }
}
