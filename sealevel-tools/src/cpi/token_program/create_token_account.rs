use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{program_error::ProgramError, program_pack::Pack, pubkey::Pubkey};

use crate::{
    account_info::{is_any_token_program_id, Account},
    cpi::{
        system_program::{try_failsafe_create_account, FailsafeCreateAccount},
        CpiAuthority, CpiPrecursor,
    },
};

/// Arguments for [try_create_token_account].
pub struct CreateTokenAccount<'a, 'b> {
    pub payer: CpiAuthority<'a, 'b>,
    pub token_account: CpiAuthority<'a, 'b>,
    pub mint: &'b NoStdAccountInfo,
    pub token_account_owner: &'b Pubkey,
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
///         NextEnumeratedAccountOptions, Account, Signer,
///     },
///     cpi::token_program::{try_create_token_account, CreateTokenAccount},
/// };
/// use solana_nostd_entrypoint::NoStdAccountInfo;
/// use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
///
/// fn process_instruction(
///      program_id: &Pubkey,
///      accounts: &[NoStdAccountInfo],
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
///     let (_, new_token_account) = try_next_enumerated_account::<Account<true>>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&new_token_addr),
///             ..Default::default()
///         },
///     )?;
///
///     // Next account must be a mint account. If you do not want to enforce that this is a read-
///     // only account, use `try_next_enumerated_account_info` instead.
///     let (_, mint_account) = try_next_enumerated_account::<Account<false>>(
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
///         mint: &mint_account,
///         token_account_owner: token_account_owner.key(),
///         opts: Default::default(), // No freeze authority specified.
///     })?;
///
///     Ok(())
/// }
/// ```
pub fn try_create_token_account<'a>(
    CreateTokenAccount {
        payer,
        token_account,
        mint,
        token_account_owner,
        opts: CreateTokenAccountOptions { token_program_id },
    }: CreateTokenAccount<'_, 'a>,
) -> Result<Account<'a, true>, ProgramError> {
    let token_program_id = match token_program_id {
        Some(program_id) => program_id,
        // Determine mint account info if the token program ID has not been provided. This info is
        // used to find the token program ID.
        None => mint.owner(),
    };

    if !is_any_token_program_id(token_program_id) {
        return Err(ProgramError::InvalidAccountData);
    }

    // Create the token account by assigning it to the token program.
    let token_account = try_failsafe_create_account(FailsafeCreateAccount {
        payer,
        to: token_account,
        space: spl_token_2022::state::Account::LEN as u64,
        program_id: token_program_id,
    })?;

    _invoke_initialize_account3_unchecked(
        token_program_id,
        &token_account,
        mint,
        token_account_owner,
    );

    Ok(token_account)
}

/// Arguments for [invoke_initialize_account3_unchecked].
pub struct InitializeAccount3<'a> {
    pub token_program_id: &'a Pubkey,
    pub account: &'a NoStdAccountInfo,
    pub mint: &'a NoStdAccountInfo,
    pub owner: &'a Pubkey,
}

/// Initialize a token account with a specific mint and owner. This method initializes a token
/// account for one of the Token programs. Only use this instruction if you have already created the
/// token account via the System program.
///
/// NOTE: It is preferred to use [try_create_token_account] instead of this method because it will
/// create the account and initialize it as a token account in one action.
pub fn invoke_initialize_account3_unchecked(
    InitializeAccount3 {
        token_program_id,
        account,
        mint,
        owner,
    }: InitializeAccount3,
) {
    _invoke_initialize_account3_unchecked(token_program_id, account, mint, owner);
}

#[inline(always)]
fn _invoke_initialize_account3_unchecked(
    token_program_id: &Pubkey,
    account: &NoStdAccountInfo,
    mint: &NoStdAccountInfo,
    owner: &Pubkey,
) {
    const IX_DATA_LEN: usize = 4 // selector
        + 32; // owner

    let mut instruction_data = [0_u8; IX_DATA_LEN];

    // Initialize account 3 selector == 18.
    instruction_data[0] = 18;
    instruction_data[1..33].copy_from_slice(&owner.to_bytes());

    CpiPrecursor {
        program_id: token_program_id,
        accounts: [account.to_meta_c(), mint.to_meta_c()],
        instruction_data,
        infos: [account.to_info_c(), mint.to_info_c()],
    }
    .invoke_signed_unchecked(&[]);
}
