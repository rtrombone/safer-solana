use alloc::vec::Vec;

use solana_nostd_entrypoint::{AccountInfoC, AccountMetaC, NoStdAccountInfo};
use solana_program::{entrypoint::ProgramResult, instruction::Instruction};

use super::{try_check_borrow_account_info, CpiInstruction};

/// Similar to [invoke_signed](solana_program::program::invoke_signed). This method performs
/// [invoke_signed_c] under the hood for a given [Instruction]. This method is useful for SDKs that
/// generate [Instruction] structs.
///
/// ### Notes
///
/// If you can generate an [InstructionC] struct or have the components to create [CpiInstruction], it
/// is recommended to use [invoke_signed_c] or [CpiInstruction::invoke_signed] instead.
///
/// [InstructionC]: solana_nostd_entrypoint::InstructionC
/// [invoke_signed_c]: super::invoke_signed_c
#[inline(always)]
pub fn invoke_signed(
    Instruction {
        program_id,
        accounts,
        data,
    }: &Instruction,
    infos: &[AccountInfoC],
    signers_seeds: &[&[&[u8]]],
) {
    let accounts = accounts
        .iter()
        .map(|meta| AccountMetaC {
            pubkey: &meta.pubkey,
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        })
        .collect::<Vec<_>>();

    CpiInstruction {
        program_id,
        accounts: &accounts,
        data,
    }
    .invoke_signed(infos, signers_seeds);
}

/// Similar to [invoke_signed](solana_program::program::invoke_signed), but performs
/// [try_check_borrow_account_info] before calling [invoke_signed].
///
/// ### Notes
///
/// Because [AccountInfoC] does not have any borrow attempt methods, this method has to allocate
/// heap memory to create an [AccountInfoC] for each [NoStdAccountInfo] in the slice. It would be
/// more efficient to use [invoke_signed] or [invoke_signed_c] if you know that all account infos
/// can be borrowed according to their write privileges.
///
/// [invoke_signed_c]: super::invoke_signed_c
#[inline(always)]
pub fn try_invoke_signed(
    instruction: &Instruction,
    account_infos: &[NoStdAccountInfo],
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    account_infos
        .iter()
        .try_for_each(try_check_borrow_account_info)?;

    invoke_signed(
        instruction,
        &account_infos
            .iter()
            .map(|account| account.to_info_c())
            .collect::<Vec<_>>(),
        signers_seeds,
    );
    Ok(())
}
