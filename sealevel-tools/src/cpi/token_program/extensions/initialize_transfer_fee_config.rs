use core::mem::size_of;

use crate::{cpi::CpiInstruction, entrypoint::NoStdAccountInfo, pubkey::Pubkey};

/// Arguments for the initialize transfer fee config instruction on the specified Token program,
/// which establishes a fee to be withheld whenever someone transfers tokens between token accounts.
/// This instruction must be called before a mint is initialized.
pub struct InitializeTransferFeeConfig<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub config_authority: Option<&'a Pubkey>,
    pub withdraw_withheld_authority: Option<&'a Pubkey>,
    pub basis_points: u16,
    pub maximum_fee: u64,
}

impl<'a> InitializeTransferFeeConfig<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            mint,
            config_authority,
            withdraw_withheld_authority,
            basis_points,
            maximum_fee,
        } = self;

        const IX_DATA_LEN: usize = {
            size_of::<u8>() // token instruction selector
            + size_of::<u8>() // extension selector
            + size_of::<u8>() // config_authority.is_some()
            + size_of::<Pubkey>() // config_authority
            + size_of::<u8>() // withdraw_withheld_authority.is_some()
            + size_of::<Pubkey>() // withdraw_withheld_authority
            + size_of::<u16>() // basis_points
            + size_of::<u64>() // maximum_fee
        };

        let mut instruction_data = [0; IX_DATA_LEN];

        // Transfer fee extension selector == 26.
        instruction_data[0] = 26;

        // Update conditionally based on optional authorities. Starting at index == 2 because the
        // initialize selector is zero.
        let mut index = 2;

        if let Some(authority) = config_authority {
            instruction_data[index] = 1;
            index += size_of::<u8>();
            instruction_data[index..(index + 32)].copy_from_slice(&authority.to_bytes());
            index += size_of::<Pubkey>();
        } else {
            index += size_of::<u8>();
        }
        if let Some(authority) = withdraw_withheld_authority {
            instruction_data[index] = 1;
            index += size_of::<u8>();
            instruction_data[index..(index + 32)].copy_from_slice(&authority.to_bytes());
            index += size_of::<Pubkey>();
        } else {
            index += size_of::<u8>();
        }
        instruction_data[index..(index + 2)].copy_from_slice(&basis_points.to_le_bytes());
        index += size_of::<u16>();
        instruction_data[index..(index + 8)].copy_from_slice(&maximum_fee.to_le_bytes());

        CpiInstruction {
            program_id: token_program_id,
            accounts: &[mint.to_meta_c()],
            data: &instruction_data,
        }
        .invoke_signed(&[mint.to_info_c()], &[]);
    }
}
