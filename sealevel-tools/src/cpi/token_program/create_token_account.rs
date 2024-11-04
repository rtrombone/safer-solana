use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{program_error::ProgramError, program_pack::Pack, pubkey::Pubkey};

use crate::{
    account_info::{is_any_token_program_id, Account},
    cpi::{system_program::CreateAccount, CpiAuthority, CpiInstruction},
};

/// Arguments to create a token account for a specific mint with a specified owner. This method
/// creates an account for one of the Token programs and initializes it as a token account.
///
/// ### Example
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, try_next_enumerated_account_info, AnyTokenProgram,
///         NextEnumeratedAccountOptions, Payer, WritableAccount,
///     },
///     cpi::token_program::CreateTokenAccount,
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
///         try_next_enumerated_account::<Payer>(&mut accounts_iter, Default::default())?;
///
///     let (new_token_addr, new_token_bump) =
///         Pubkey::find_program_address(&[b"token"], program_id);
///
///     // Next account must be writable data account matching PDA address.
///     let (_, new_token_account) = try_next_enumerated_account::<WritableAccount>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&new_token_addr),
///             ..Default::default()
///         },
///     )?;
///
///     // Next account must be a mint account. If you do not want to enforce that this is a read-
///     // only account, use `try_next_enumerated_account_info` instead.
///     let (_, mint_account) = try_next_enumerated_account::<WritableAccount>(
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
///     CreateTokenAccount {
///         payer: payer.as_cpi_authority(),
///         token_account: new_token_account.as_cpi_authority(Some(&[b"token", &[new_token_bump]])),
///         mint: &mint_account,
///         token_account_owner: token_account_owner.key(),
///         opts: Default::default(),
///     }
///     .try_into_invoke()?;
///
///     Ok(())
/// }
/// ```
pub struct CreateTokenAccount<'a, 'b> {
    pub payer: CpiAuthority<'a, 'b>,
    pub token_account: CpiAuthority<'a, 'b>,
    pub mint: &'a NoStdAccountInfo,
    pub token_account_owner: &'b Pubkey,
    pub opts: CreateTokenAccountOptions,
}

/// Optional arguments for [CreateTokenAccount].
#[derive(Debug, Default)]
pub struct CreateTokenAccountOptions {
    /// If false, the initialize immutable owner instruction will be used to prevent the owner from
    /// being changed in the future. This argument does not apply for the Legacy Token program.
    pub mutable_owner: bool,

    /// Override lamports sent to the token account. This argument can be useful in case more
    /// extensions will be added in subsequent CPI calls. Setting this to None will use the minimum
    /// required for rent.
    pub lamports: Option<u64>,
}

impl<'a, 'b> CreateTokenAccount<'a, 'b> {
    /// Try to consume arguments to perform CPI calls.
    #[inline(always)]
    pub fn try_into_invoke(self) -> Result<Account<'a, true>, ProgramError> {
        let Self {
            payer,
            token_account,
            mint,
            token_account_owner,
            opts:
                CreateTokenAccountOptions {
                    mutable_owner,
                    lamports,
                },
        } = self;

        let token_program_id = mint.owner();

        if !is_any_token_program_id(token_program_id) {
            return Err(ProgramError::InvalidAccountData);
        }

        let token_account = if token_program_id == &spl_token::ID || mutable_owner {
            // Create the token account by assigning it to the token program.
            CreateAccount {
                payer,
                to: token_account,
                program_id: token_program_id,
                space: Some(spl_token_2022::state::Account::LEN),
                lamports,
            }
            .try_into_invoke()?
        } else {
            // Create the token account by assigning it to the token program.
            let token_account = CreateAccount {
                payer,
                to: token_account,
                program_id: token_program_id,
                // Caching the size of a token account with immutable owner to avoid having to
                // invoke the get account data size instruction.
                space: Some(170),
                lamports,
            }
            .try_into_invoke()?;

            super::_invoke_initialize_immutable_owner(token_program_id, &token_account);

            token_account
        };

        _invoke_initialize_account3(token_program_id, &token_account, mint, token_account_owner);

        Ok(token_account)
    }
}

/// Arguments for the initialize token account instruction (version 3), which initializes a token
/// account for one of the Token programs. Only use this instruction if you have already created the
/// token account via the System program.
///
/// ### Notes
///
/// It is preferred to use [CreateTokenAccount] instead of this method because it will create
/// the account and initialize it as a token account in one action.
pub struct InitializeAccount3<'a> {
    pub token_program_id: &'a Pubkey,
    pub account: &'a NoStdAccountInfo,
    pub mint: &'a NoStdAccountInfo,
    pub owner: &'a Pubkey,
}

impl<'a> InitializeAccount3<'a> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            account,
            mint,
            owner,
        } = self;

        _invoke_initialize_account3(token_program_id, account, mint, owner);
    }
}

#[inline(always)]
fn _invoke_initialize_account3(
    token_program_id: &Pubkey,
    account: &NoStdAccountInfo,
    mint: &NoStdAccountInfo,
    owner: &Pubkey,
) {
    // Initialize account 3 selector == 18.
    let instruction_data = super::serialize_authority_instruction_data(18, owner);

    CpiInstruction {
        program_id: token_program_id,
        accounts: &[account.to_meta_c(), mint.to_meta_c()],
        data: &instruction_data,
    }
    .invoke_signed(&[account.to_info_c(), mint.to_info_c()], &[]);
}
