//! CPI for either SPL Token or Token Extensions programs.
//!
//! ### Notes
//!
//! This module does not have optimized CPI calls for every instruction. For any instruction you
//! need for your program that does not exist here, please use [invoke_signed] with instruction
//! builders found in the [spl_token_2022::instruction] module.
//!
//! See detailed examples of how to perform CPI with [CreateMint], [CreateTokenAccount], [MintTo]
//! and [Transfer].
//!
//! [invoke_signed]: crate::cpi::invoke_signed

mod approve;
mod burn;
mod close_account;
mod create_mint;
mod create_token_account;
pub mod extensions;
mod freeze_account;
#[cfg(feature = "alloc")]
mod get_account_data_size;
mod mint_to;
mod revoke;
mod set_authority;
mod sync_native;
mod thaw_account;
mod transfer;

pub use approve::*;
pub use burn::*;
pub use close_account::*;
pub use create_mint::*;
pub use create_token_account::*;
pub use freeze_account::*;
#[cfg(feature = "alloc")]
pub use get_account_data_size::*;
pub use mint_to::*;
pub use revoke::*;
pub use set_authority::*;
pub use sync_native::*;
pub use thaw_account::*;
pub use transfer::*;

pub use spl_token_2022::{extension::ExtensionType, instruction::AuthorityType};

use core::mem::size_of;

use solana_program_pack::Pack;

use crate::{error::SealevelToolsError, pubkey::Pubkey};

const BASE_WITH_EXTENSIONS_LEN: usize = {
    spl_token_2022::state::Account::LEN // base size
    + size_of::<u8>() // account type size
};

const EMPTY_EXTENSION_LEN: usize = {
    size_of::<u16>() // type
    + size_of::<u16>() // length
};

const ERROR_EXPECTED_TOKEN_PROGRAM: SealevelToolsError<'static> =
    SealevelToolsError::Cpi(&["Expected legacy SPL Token or Token Extensions program as ID"]);
const ERROR_EXTENSIONS_UNSUPPORTED: SealevelToolsError<'static> =
    SealevelToolsError::Cpi(&["Extensions only supported with SPL Token Extensions program"]);

const IX_AMOUNT_DATA_LEN: usize = {
    size_of::<u8>() // selector
    + size_of::<u64>() // amount
};

#[inline(always)]
fn serialize_amount_instruction_data(selector: u8, amount: u64) -> [u8; IX_AMOUNT_DATA_LEN] {
    let mut instruction_data = [0; IX_AMOUNT_DATA_LEN];

    instruction_data[0] = selector;
    instruction_data[1..9].copy_from_slice(&amount.to_le_bytes());

    instruction_data
}

const IX_CHECKED_AMOUNT_DATA_LEN: usize = {
    1 // selector
    + size_of::<u64>() // amount
    + size_of::<u8>() // decimals
};

#[inline(always)]
fn serialize_checked_amount_instruction_data(
    selector: u8,
    amount: u64,
    decimals: u8,
) -> [u8; IX_CHECKED_AMOUNT_DATA_LEN] {
    let mut instruction_data = [0; IX_CHECKED_AMOUNT_DATA_LEN];

    instruction_data[0] = selector;
    instruction_data[1..9].copy_from_slice(&amount.to_le_bytes());
    instruction_data[9] = decimals;

    instruction_data
}

const IX_PUBKEY_DATA_LEN: usize = {
    1 // selector
    + size_of::<Pubkey>() // owner or delegate
};

#[inline(always)]
fn serialize_authority_instruction_data(
    selector: u8,
    authority: &Pubkey,
) -> [u8; IX_PUBKEY_DATA_LEN] {
    let mut instruction_data = [0; IX_PUBKEY_DATA_LEN];

    instruction_data[0] = selector;
    instruction_data[1..33].copy_from_slice(&authority.to_bytes());

    instruction_data
}

#[cfg(test)]
mod test {
    use spl_token_2022::instruction::TokenInstruction;

    use super::*;

    #[test]
    fn test_serialize_amount_instruction_data() {
        let amount = 69;

        #[allow(deprecated)]
        let expected_transfer = TokenInstruction::Transfer { amount };

        for (selector, instruction) in [
            (3, expected_transfer),
            (4, TokenInstruction::Approve { amount }),
            (7, TokenInstruction::MintTo { amount }),
            (8, TokenInstruction::Burn { amount }),
            (23, TokenInstruction::AmountToUiAmount { amount }),
        ]
        .into_iter()
        {
            assert_eq!(
                TokenInstruction::unpack(&serialize_amount_instruction_data(selector, amount))
                    .unwrap(),
                instruction,
                "Mismatch with selector {}",
                selector
            );
        }
    }

    #[test]
    fn test_serialize_checked_amount_instruction_data() {
        let amount = 69;
        let decimals = 4;

        for (selector, instruction) in [
            (12, TokenInstruction::TransferChecked { amount, decimals }),
            (13, TokenInstruction::ApproveChecked { amount, decimals }),
            (14, TokenInstruction::MintToChecked { amount, decimals }),
            (15, TokenInstruction::BurnChecked { amount, decimals }),
        ]
        .into_iter()
        {
            assert_eq!(
                TokenInstruction::unpack(&serialize_checked_amount_instruction_data(
                    selector, amount, decimals
                ))
                .unwrap(),
                instruction,
                "Mismatch with selector {}",
                selector
            );
        }
    }

    #[test]
    fn test_serialize_authority_instruction_data() {
        let authority = Pubkey::new_unique();

        for (selector, instruction) in [
            (
                16,
                TokenInstruction::InitializeAccount2 { owner: authority },
            ),
            (
                18,
                TokenInstruction::InitializeAccount3 { owner: authority },
            ),
            (
                35,
                TokenInstruction::InitializePermanentDelegate {
                    delegate: authority,
                },
            ),
        ]
        .into_iter()
        {
            assert_eq!(
                TokenInstruction::unpack(&serialize_authority_instruction_data(
                    selector, &authority
                ))
                .unwrap(),
                instruction,
                "Mismatch with selector {}",
                selector
            );
        }
    }
}
