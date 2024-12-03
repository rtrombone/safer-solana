use core::mem::size_of;

use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize close authority instruction on the specified Token program, which
/// initializes the close authority of a mint. This instruction must be called before a mint is
/// initialized.
pub struct InitializeMintCloseAuthority<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub authority: Option<&'a Pubkey>,
}

impl<'a> InitializeMintCloseAuthority<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            authority,
        } = self;

        const IX_DATA_LEN: usize = {
            size_of::<u8>() // token instruction selector
            + size_of::<u8>() // authority.is_some()
            + size_of::<Pubkey>() // authority
        };

        let mut instruction_data = [0; IX_DATA_LEN];

        // Initialize close authority selector == 25.
        instruction_data[0] = 25;
        if let Some(authority) = authority {
            instruction_data[1] = 1;
            instruction_data[2..34].copy_from_slice(&authority.to_bytes());
        }

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            data: &instruction_data,
        }
        .invoke_signed(&[mint.to_info_c()], &[]);
    }
}
