use solana_program::{
    account_info::AccountInfo, program::invoke, program_error::ProgramError, program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token_2022::instruction::initialize_account3;

use crate::{
    account_info::DataAccount,
    cpi::{
        system_program::{try_create_account, CreateAccount},
        CpiAccount, CpiAuthority,
    },
    error::SealevelToolsError,
};

/// Arguments for [try_create_token_account].
#[derive(Debug)]
pub struct CreateTokenAccount<'a, 'b, 'c> {
    pub payer: CpiAuthority<'a, 'b, 'c>,
    pub token_account: CpiAuthority<'a, 'b, 'c>,
    pub mint: CpiAccount<'a, 'c>,
    pub token_account_owner: CpiAccount<'a, 'c>,
    pub account_infos: &'c [AccountInfo<'a>],
    pub opts: CreateTokenAccountOptions<'a>,
}

/// Optional arguments for [try_create_token_account].
#[derive(Debug, Default)]
pub struct CreateTokenAccountOptions<'a> {
    pub token_program_id: Option<&'a Pubkey>,
}

/// Create a token account for a specific mint with a specified owner. This method creates an
/// account for one of the Token programs and initializes it as a token account.
///
/// ### Example
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, try_next_enumerated_account_info, AnyTokenProgram,
///         NextEnumeratedAccountOptions, DataAccount, Signer,
///     },
///     cpi::{
///         token_program::{try_create_token_account, CreateTokenAccount},
///         CpiAccount,
///     },
/// };
/// use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
///
/// fn process_instruction(
///      program_id: &Pubkey,
///      accounts: &[AccountInfo],
///      instruction_data: &[u8],
/// ) -> ProgramResult {
///     let mut accounts_iter = accounts.iter().enumerate();
///
///     // Next account must writable signer (A.K.A. our payer).
///     let (_, payer) =
///         try_next_enumerated_account::<Signer<true>>(&mut accounts_iter, Default::default())?;
///
///     let (new_token_addr, new_token_bump) =
///         Pubkey::find_program_address(&[b"token"], program_id);
///
///     // Next account must be writable data account matching PDA address.
///     let (_, new_token_account) = try_next_enumerated_account::<DataAccount<true>>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&new_token_addr),
///             ..Default::default()
///         },
///     )?;
///
///     // Next account must be a mint account. If you do not want to enforce that this is a read-
///     // only account, use `try_next_enumerated_account_info` instead.
///     let (_, mint_account) = try_next_enumerated_account::<DataAccount<false>>(
///         &mut accounts_iter,
///         Default::default()
///     )?;
///
///     // Next account is the owner.
///     let (_, token_account_owner) =
///         try_next_enumerated_account_info(&mut accounts_iter, Default::default())?;
///
///     // Next account is which token program to use.
///     let (_, token_program) =
///         try_next_enumerated_account::<AnyTokenProgram>(&mut accounts_iter, Default::default())?;
///
///     try_create_token_account(CreateTokenAccount {
///         payer: payer.as_cpi_authority(),
///         token_account: new_token_account.as_cpi_authority(Some(&[b"token", &[new_token_bump]])),
///         mint: mint_account.as_cpi_account(),
///         token_account_owner: CpiAccount::Info(&token_account_owner),
///         account_infos: accounts,
///         opts: Default::default(), // No freeze authority specified.
///     })?;
///
///     Ok(())
/// }
/// ```
pub fn try_create_token_account<'a, 'c>(
    CreateTokenAccount {
        payer,
        token_account,
        mint,
        token_account_owner,
        account_infos,
        opts: CreateTokenAccountOptions { token_program_id },
    }: CreateTokenAccount<'a, '_, 'c>,
) -> Result<DataAccount<'a, 'c, true>, ProgramError> {
    let token_program_id = match token_program_id {
        Some(program_id) => program_id,
        // Determine mint account info if the token program ID has not been provided. This info is
        // used to find the token program ID.
        None => match &mint {
            CpiAccount::Key(mint_key) => account_infos
                .iter()
                .find(|info| info.key == *mint_key)
                .map(|info| info.owner)
                .ok_or_else(|| {
                    SealevelToolsError::Cpi("token_program", format!("Cannot find mint {mint_key}"))
                })?,
            CpiAccount::Info(info) => info.owner,
        },
    };

    // Create the token account by assigning it to the token program.
    let token_account = try_create_account(CreateAccount {
        payer,
        to: token_account,
        space: spl_token_2022::state::Account::LEN as u64,
        program_id: token_program_id,
        account_infos,
    })?;

    // NOTE: Token program ID passed into this method is checked against either token program IDs.
    // This method will revert with IncorrectProgramId if the token program ID is not correct.
    let instruction = initialize_account3(
        token_program_id,
        token_account.key,
        mint.key(),
        token_account_owner.key(),
    )?;

    invoke(&instruction, account_infos)?;

    Ok(token_account)
}
