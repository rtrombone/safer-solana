#[cfg(feature = "borsh")]
mod borsh;

#[cfg(feature = "borsh")]
pub use borsh::*;

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::Instruction,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::{allocate, assign, create_account, transfer},
    sysvar::Sysvar,
};

use crate::{account_info::DataAccount, error::SealevelToolsError};

pub enum ToAccount<'a, 'b> {
    Key(&'b Pubkey),
    Info(&'b AccountInfo<'a>),
}

impl<'a, 'b> From<&'b Pubkey> for ToAccount<'a, 'b> {
    fn from(pubkey: &'b Pubkey) -> Self {
        ToAccount::Key(pubkey)
    }
}

impl<'a, 'b> From<&'b AccountInfo<'a>> for ToAccount<'a, 'b> {
    fn from(info: &'b AccountInfo<'a>) -> Self {
        ToAccount::Info(info)
    }
}

pub struct CreateAccount<'a, 'b, 'c> {
    pub from_pubkey: &'c Pubkey,
    pub to: ToAccount<'a, 'c>,
    pub space: u64,
    pub program_id: &'c Pubkey,
    pub account_infos: &'c [AccountInfo<'a>],
    pub from_signer_seeds: Option<&'c [&'b [u8]]>,
    pub to_signer_seeds: Option<&'c [&'b [u8]]>,
}

pub fn try_create_account<'a, 'c>(
    CreateAccount {
        from_pubkey,
        to,
        account_infos,
        space,
        program_id,
        from_signer_seeds,
        to_signer_seeds,
    }: CreateAccount<'a, '_, 'c>,
) -> Result<DataAccount<'a, 'c, true>, ProgramError> {
    let rent_required = Rent::get().map(|rent| rent.minimum_balance(space as usize))?;
    let to_info = match to {
        ToAccount::Key(to_pubkey) => account_infos
            .iter()
            .find(|info| info.key == to_pubkey)
            .ok_or_else(|| {
                SealevelToolsError::CpiSystemProgramCreateAccount(format!(
                    "Cannot find {to_pubkey}"
                ))
            })?,
        ToAccount::Info(to_info) => to_info,
    };

    let current_lamports = to_info.lamports();

    if current_lamports == 0 {
        handle_invoke_signed_from_to(
            &create_account(from_pubkey, to_info.key, rent_required, space, program_id),
            account_infos,
            from_signer_seeds,
            to_signer_seeds,
        )?;
    } else {
        let lamport_diff = rent_required.saturating_sub(current_lamports);

        if lamport_diff != 0 {
            // Transfer remaining lamports.
            handle_invoke_signed_from_to(
                &transfer(from_pubkey, to_info.key, lamport_diff),
                account_infos,
                from_signer_seeds,
                to_signer_seeds,
            )?;
        }

        if space != 0 {
            // Allocate space.
            invoke_signed(
                &allocate(to_info.key, space),
                account_infos,
                &[to_signer_seeds.unwrap_or_default()],
            )?;
        }

        // Assign to specified program.
        invoke_signed(
            &assign(to_info.key, program_id),
            account_infos,
            &[to_signer_seeds.unwrap_or_default()],
        )?;
    }

    // We know that this account was writable, so we are safe to instantiate it like this.
    Ok(DataAccount(to_info))
}

fn handle_invoke_signed_from_to(
    instruction: &Instruction,
    account_infos: &[AccountInfo],
    from_signer_seeds: Option<&[&[u8]]>,
    to_signer_seeds: Option<&[&[u8]]>,
) -> ProgramResult {
    match (from_signer_seeds, to_signer_seeds) {
        (Some(from_signer_seeds), Some(to_signer_seeds)) => invoke_signed(
            instruction,
            account_infos,
            &[from_signer_seeds, to_signer_seeds],
        ),
        (None, Some(to_signer_seeds)) => {
            invoke_signed(instruction, account_infos, &[to_signer_seeds])
        }
        (Some(from_signer_seeds), None) => {
            invoke_signed(instruction, account_infos, &[from_signer_seeds])
        }
        (None, None) => invoke_signed(instruction, account_infos, &[]),
    }
}
