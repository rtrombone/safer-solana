use core::mem::size_of;

use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize permanent delegate instruction on the specified Token program,
/// which allows this authority for the specified mint to move tokens between any token accounts.
/// This instruction must be called before a mint is initialized.
pub struct InitializePermanentDelegate<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub delegate: &'a Pubkey,
}

impl<'a> InitializePermanentDelegate<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            delegate,
        } = self;

        const IX_DATA_LEN: usize = {
            size_of::<u8>() // token instruction selector
            + size_of::<Pubkey>() // delegate
        };

        let mut instruction_data = [0; IX_DATA_LEN];

        // Initialize permanent delegate selector == 35.
        instruction_data[0] = 35;
        instruction_data[1..33].copy_from_slice(&delegate.to_bytes());

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            data: &instruction_data,
        }
        .invoke_signed(&[mint.to_info_c()], &[]);
    }
}
