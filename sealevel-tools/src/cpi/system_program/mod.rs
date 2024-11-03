//! CPI for System program.

mod allocate;
mod assign;
mod create_account;
mod transfer;

pub use allocate::*;
pub use assign::*;
pub use create_account::*;
pub use transfer::*;

pub use solana_program::system_program::ID;

#[inline(always)]
fn _invoke_signed_from_to_unchecked<const ACCOUNT_LEN: usize, const DATA_LEN: usize>(
    precursor: super::CpiPrecursor<ACCOUNT_LEN, DATA_LEN>,
    from_signer_seeds: Option<&[&[u8]]>,
    to_signer_seeds: Option<&[&[u8]]>,
) {
    match (from_signer_seeds, to_signer_seeds) {
        (Some(from_signer_seeds), Some(to_signer_seeds)) => {
            precursor.invoke_signed_unchecked(&[from_signer_seeds, to_signer_seeds])
        }
        (None, Some(to_signer_seeds)) => precursor.invoke_signed_unchecked(&[to_signer_seeds]),
        (Some(from_signer_seeds), None) => precursor.invoke_signed_unchecked(&[from_signer_seeds]),
        (None, None) => precursor.invoke_signed_unchecked(&[]),
    };
}
