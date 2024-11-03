use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{program_error::ProgramError, program_pack::Pack, pubkey::Pubkey};
use spl_token_2022::state::Mint;

use crate::{
    account_info::{is_any_token_program_id, Account},
    cpi::{
        system_program::{try_failsafe_create_account, FailsafeCreateAccount},
        CpiAuthority, CpiPrecursor,
    },
};

/// Arguments for [try_create_mint].
pub struct CreateMint<'a, 'b> {
    pub token_program_id: &'b Pubkey,
    pub payer: CpiAuthority<'a, 'b>,
    pub mint: CpiAuthority<'a, 'b>,
    pub mint_authority: &'b Pubkey,
    pub decimals: u8,
    pub opts: CreateMintOptions<'b>,
}

/// Optional arguments for [try_create_mint].
#[derive(Default)]
pub struct CreateMintOptions<'a> {
    pub freeze_authority: Option<&'a Pubkey>,
}

/// Create a mint account. This method creates an account for one of the Token programs and
/// initializes it as a mint.
///
/// ### Example
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, try_next_enumerated_account_info, AnyTokenProgram,
///         NextEnumeratedAccountOptions, Account, Signer,
///     },
///     cpi::token_program::{try_create_mint, CreateMint},
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
///     let (new_mint_addr, new_mint_bump) =
///         Pubkey::find_program_address(&[b"mint"], program_id);
///
///     // Next account must be writable data account matching PDA address.
///     let (_, new_mint_account) = try_next_enumerated_account::<Account<true>>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&new_mint_addr),
///             ..Default::default()
///         },
///     )?;
///
///     // Next account is the mint authority.
///     let (_, mint_authority) =
///         try_next_enumerated_account_info(&mut accounts_iter, Default::default())?;
///
///     // Next account is which token program to use.
///     let (_, token_program) =
///         try_next_enumerated_account::<AnyTokenProgram>(&mut accounts_iter, Default::default())?;
///
///     try_create_mint(CreateMint {
///         token_program_id: token_program.key(),
///         payer: payer.as_cpi_authority(),
///         mint: new_mint_account.as_cpi_authority(Some(&[b"mint", &[new_mint_bump]])),
///         mint_authority: mint_authority.key(),
///         decimals: 9,
///         opts: Default::default(), // No freeze authority specified.
///     })?;
///
///     Ok(())
/// }
/// ```
pub fn try_create_mint<'b>(
    CreateMint {
        token_program_id,
        payer,
        mint,
        mint_authority,
        decimals,
        opts: CreateMintOptions { freeze_authority },
    }: CreateMint<'_, 'b>,
) -> Result<Account<'b, true>, ProgramError> {
    if !is_any_token_program_id(token_program_id) {
        return Err(ProgramError::InvalidAccountData);
    }

    // First create the mint account by assigning it to the token program.
    let mint_account = try_failsafe_create_account(FailsafeCreateAccount {
        payer,
        to: mint,
        space: Mint::LEN as u64,
        program_id: token_program_id,
    })?;

    _invoke_initialize_mint2_unchecked(
        token_program_id,
        &mint_account,
        mint_authority,
        freeze_authority,
        decimals,
    );

    Ok(mint_account)
}

/// Arguments for [invoke_initialize_mint2_unchecked].
pub struct InitializeMint2<'a> {
    pub token_program_id: &'a Pubkey,
    pub mint: &'a NoStdAccountInfo,
    pub mint_authority: &'a Pubkey,
    pub freeze_authority: Option<&'a Pubkey>,
    pub decimals: u8,
}

/// Initialize a mint account. This method initializes a mint account for one of the Token programs.
/// Only use this instruction if you have already created the mint account via the System program.
///
/// NOTE: It is preferred to use [try_create_mint] instead of this method because it will create the
/// account and initialize it as a mint in one action.
pub fn invoke_initialize_mint2_unchecked(
    InitializeMint2 {
        token_program_id,
        mint,
        mint_authority,
        freeze_authority,
        decimals,
    }: InitializeMint2,
) {
    _invoke_initialize_mint2_unchecked(
        token_program_id,
        mint,
        mint_authority,
        freeze_authority,
        decimals,
    );
}

#[inline(always)]
fn _invoke_initialize_mint2_unchecked(
    token_program_id: &Pubkey,
    mint: &NoStdAccountInfo,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) {
    const IX_DATA_LEN: usize = 1 // selector
        + 1 // decimals
        + 32 // mint_authority
        + 1 + 32; // freeze_authority

    let mut instruction_data = [0; IX_DATA_LEN];

    // Initialize mint 2 selector == 20.
    instruction_data[0] = 20;
    instruction_data[1] = decimals;
    instruction_data[2..34].copy_from_slice(&mint_authority.to_bytes());

    if let Some(freeze_authority) = freeze_authority {
        instruction_data[34] = 1;
        instruction_data[35..67].copy_from_slice(&freeze_authority.to_bytes());
    }

    CpiPrecursor {
        program_id: token_program_id,
        accounts: [mint.to_meta_c()],
        instruction_data,
        infos: [mint.to_info_c()],
    }
    .invoke_signed_unchecked(&[]);
}
