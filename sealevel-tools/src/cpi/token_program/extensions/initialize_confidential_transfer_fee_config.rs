use core::mem::size_of;

use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize confidential transfer fee config instruction on the specified Token
/// program, which establishes a fee authority and ElGamal pubkey for confidential transfer fees.
/// This instruction must be called before a mint is initialized.
#[derive(Clone, PartialEq, Eq)]
pub struct InitializeConfidentialTransferFeeConfig<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub authority: Option<&'a Pubkey>,
    pub withdraw_withheld_authority_elgamal: &'a [u8; 32],
}

impl<'a> InitializeConfidentialTransferFeeConfig<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            authority,
            withdraw_withheld_authority_elgamal,
        } = self;

        let authority = super::unwrap_or_default_pubkey(authority);

        const IX_DATA_LEN: usize = {
            size_of::<u8>() // token instruction selector
            + size_of::<u8>() // pointer instruction selector
            + size_of::<Pubkey>() // authority
            + 32 // withdraw_withheld_authority_elgamal
        };

        let mut instruction_data = [0; IX_DATA_LEN];

        // Initialize confidential transfer fee config selector == 37.
        instruction_data[0] = 37;

        // Initialize extension pointer selector == 0, so no need to set it.
        instruction_data[2..34].copy_from_slice(&authority.to_bytes());
        instruction_data[34..66].copy_from_slice(withdraw_withheld_authority_elgamal);

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            data: &instruction_data,
        }
        .invoke_signed(&[mint.to_info_c()], &[]);
    }
}
