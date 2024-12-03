use core::mem::size_of;

use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize confidential transfer instruction on the specified Token program,
/// which establishes an authority and ElGamal pubkey for confidential transfers. This instruction
/// must be called before a mint is initialized.
pub struct InitializeConfidentialTransfer<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub authority: Option<&'a Pubkey>,
    pub auto_approve_new_accounts: bool,
    pub auditor_elgamal: Option<&'a [u8; 32]>,
}

impl<'a> InitializeConfidentialTransfer<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            authority,
            auto_approve_new_accounts,
            auditor_elgamal,
        } = self;

        let authority = super::unwrap_or_default_pubkey(authority);
        let auditor_elgamal = auditor_elgamal.unwrap_or(&[0; 32]);

        const IX_DATA_LEN: usize = {
            size_of::<u8>() // token instruction selector
            + size_of::<u8>() // pointer instruction selector
            + size_of::<Pubkey>() // authority
            + size_of::<bool>() // auto_approve_new_accounts
            + 32 // auditor_elgamal
        };

        let mut instruction_data = [0; IX_DATA_LEN];

        // Initialize confidential transfer selector == 27.
        instruction_data[0] = 27;

        // Initialize extension pointer selector == 0, so no need to set it.
        instruction_data[2..34].copy_from_slice(&authority.to_bytes());
        instruction_data[34] = u8::from(auto_approve_new_accounts);
        instruction_data[35..67].copy_from_slice(auditor_elgamal);

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            data: &instruction_data,
        }
        .invoke_signed(&[mint.to_info_c()], &[]);
    }
}
