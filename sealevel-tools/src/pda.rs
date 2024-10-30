//! Utilities for PDAs (program-derived accounts) like deriving PDA addresses.

use solana_program::pubkey::Pubkey;

/// Simple trait to derive a PDA address for a type given some seeds.
///
/// ### Example
///
/// ```
/// use sealevel_tools::pda::{DeriveAddress, ToSeed};
/// use solana_program::{declare_id, pubkey::Pubkey};
///
/// declare_id!("ThingProcessor11111111111111111111111111111");
///
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct IdentifiableThing {
///     id: u32,
///     data: u64
/// }
///
/// impl IdentifiableThing {
///     pub const SEED: &'static [u8] = b"thing";
/// }
///
/// impl DeriveAddress for IdentifiableThing {
///     type Seeds<'a> = u32;
///
///     fn find_program_address(id: Self::Seeds<'_>) -> (Pubkey, u8) {
///         Pubkey::find_program_address(&[IdentifiableThing::SEED, &id.to_be_bytes()], &ID)
///     }
///
///     fn create_program_address(id: Self::Seeds<'_>, bump_seed: u8) -> Option<Pubkey> {
///         Pubkey::create_program_address(
///             &[
///                 IdentifiableThing::SEED,
///                 &id.to_be_bytes(),
///                 &[bump_seed]
///             ],
///             &ID
///         )
///         .ok()
///     }
/// }
///
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct ThingCollateral {
///     thing_id: u32,
///     mint: Pubkey,
///     data: u64
/// }
///
/// impl ThingCollateral {
///     pub const SEED: &'static [u8] = b"thing_collateral";
/// }
///
/// impl DeriveAddress for ThingCollateral {
///     type Seeds<'a> = (
///         u32, // thing_id
///         &'a Pubkey, // mint
///     );
///
///     fn find_program_address(seeds: Self::Seeds<'_>) -> (Pubkey, u8) {
///         let (thing_id, mint) = seeds;
///         Pubkey::find_program_address(
///             &[
///                 ThingCollateral::SEED,
///                 &thing_id.to_seed(),
///                 mint.as_ref()
///             ],
///             &ID
///         )
///     }
///
///     fn create_program_address(seeds: Self::Seeds<'_>, bump_seed: u8) -> Option<Pubkey> {
///         let (thing_id, mint) = seeds;
///         Pubkey::create_program_address(
///             &[
///                 ThingCollateral::SEED,
///                 &thing_id.to_seed(),
///                 mint.as_ref(),
///                 &[bump_seed]
///             ],
///             &ID
///         )
///         .ok()
///     }
/// }
/// ```
pub trait DeriveAddress {
    /// Seeds required to derive the address. This type can be a primitive, struct or tuple, as long
    /// as its definition makes sense for the type implementing this trait.
    type Seeds<'a>;

    /// Using [Pubkey::find_program_address], implement a custom address finder for a PDA.
    fn find_program_address(seeds: Self::Seeds<'_>) -> (Pubkey, u8);

    /// Using [Pubkey::create_program_address], implement a custom address creator for a PDA.
    fn create_program_address(seeds: Self::Seeds<'_>, bump_seed: u8) -> Option<Pubkey>;
}

/// Trait to convert an integer to a seed consistently to big-endian bytes.
pub trait ToSeed {
    type Bytes;

    fn to_seed(self) -> Self::Bytes;
}

macro_rules! impl_to_seed {
    ($($t:ty),*) => {
        $(
            impl ToSeed for $t {
                type Bytes = [u8; core::mem::size_of::<$t>()];

                fn to_seed(self) -> Self::Bytes {
                    self.to_be_bytes()
                }
            }
        )*
    };
}

impl_to_seed!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
