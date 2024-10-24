//! Discriminator generation for program accounts, events, and instructions.

/// Discriminator generated either by user-defined or by specific hashing function (where total hash
/// output is 256 bits). These discriminators can be used for discriminating against serialized
/// program accounts, serialized events, and instructions (as selectors for specific program
/// instructions).
///
/// Only Keccak, Sha2 and Sha3 hashing are supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Discriminator<'a, const N: usize> {
    Defined([u8; N]),

    // Using keccak hasher.
    Keccak(&'a [u8]),

    // Using sha2 hasher. Discrminators used in anchor-lang and spl-discriminator crates use the
    // first 8 bytes of this type.
    Sha2(&'a [u8]),

    /// Using sha3 hasher.
    Sha3(&'a [u8]),
}

impl<'a, const N: usize> Discriminator<'a, N> {
    /// Be careful when using this method because there is no const constraint that prevents N to be
    /// larger than a computed digest's length (32 bytes). If N is larger, the output will be padded
    /// to the right with zeros.
    pub const fn to_bytes(self) -> [u8; N] {
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

        let mut inner = [0; N];
        let mut i = 0;
        loop {
            if i >= N || i >= digest.len() {
                break;
            }

            inner[i] = digest[i];
            i += 1;
        }

        inner
    }
}

pub trait Discriminate<const N: usize> {
    const DISCRIMINATOR: [u8; N];
}

impl<const N: usize, T: Discriminate<N>> Discriminate<N> for Box<T> {
    const DISCRIMINATOR: [u8; N] = T::DISCRIMINATOR;
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
