use alloc::vec::Vec;

use crate::{
    cpi::{checked_return_data, CpiInstruction},
    entrypoint::NoStdAccountInfo,
    pubkey::Pubkey,
};

use super::ExtensionType;

/// Arguments for the get account data size instruction on the specified Token program, which
/// retrieves the size of a token account data for a mint.
#[derive(Clone, PartialEq)]
pub struct GetAccountDataSize<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub extensions: &'a [ExtensionType],
}

impl<'a> GetAccountDataSize<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_return_data(self) -> usize {
        let Self {
            token_program_id,
            mint,
            extensions,
        } = self;

        const IX_DATA_LEN_FIXED: usize = core::mem::size_of::<u8>(); // selector

        // Get account data size selector == 21.
        let mut instruction_data =
            Vec::with_capacity(IX_DATA_LEN_FIXED + core::mem::size_of::<u16>() * extensions.len());
        instruction_data.push(21);
        extensions
            .iter()
            .for_each(|extension| instruction_data.extend_from_slice(&<[u8; 2]>::from(*extension)));

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            data: &instruction_data,
        }
        .invoke_signed(&[mint.to_info_c()], &[]);

        u64::from_le_bytes(checked_return_data().unwrap().1) as usize
    }
}
