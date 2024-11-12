use core::mem::size_of;

use crate::{
    account_info::{is_any_token_program_id, Account},
    cpi::{system_program::CreateAccount, CpiAuthority, CpiInstruction},
    entrypoint::NoStdAccountInfo,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    spl_token_2022::{
        extension::{
            non_transferable::NonTransferable, transfer_fee::TransferFeeConfig,
            transfer_hook::TransferHook, BaseStateWithExtensions, PodStateWithExtensions,
        },
        pod::PodMint,
    },
};

use super::EMPTY_EXTENSION_LEN;

/// Arguments to create a token account for a specific mint with a specified owner. This method
/// creates an account for one of the Token programs and initializes it as a token account.
///
/// ### Example
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, try_next_enumerated_account_info, TokenProgram,
///         EnumeratedAccountConstraints, Payer, WritableAccount,
///     },
///     cpi::token_program::CreateTokenAccount,
///     entrypoint::{NoStdAccountInfo, ProgramResult},
///     pubkey::Pubkey,
/// };
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
///         EnumeratedAccountConstraints {
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
///         try_next_enumerated_account::<TokenProgram>(&mut accounts_iter, Default::default())?;
///
///     CreateTokenAccount {
///         payer: payer.as_cpi_authority(),
///         token_account: new_token_account.as_cpi_authority(Some(&[b"token", &[new_token_bump]])),
///         mint: &mint_account,
///         token_account_owner: token_account_owner.key(),
///         immutable_owner: true,
///     }
///     .try_into_invoke()?;
///
///     Ok(())
/// }
/// ```
pub struct CreateTokenAccount<'a, 'b: 'a> {
    pub payer: CpiAuthority<'a, 'b>,
    pub token_account: CpiAuthority<'a, 'b>,
    pub mint: &'b NoStdAccountInfo,
    pub token_account_owner: &'a Pubkey,
    pub immutable_owner: bool,
}

impl<'a, 'b: 'a> CreateTokenAccount<'a, 'b> {
    /// Try to consume arguments to perform CPI calls.
    #[inline(always)]
    pub fn try_into_invoke(self) -> Result<Account<'a, true>, ProgramError> {
        let Self {
            payer,
            token_account,
            mint,
            token_account_owner,
            immutable_owner,
        } = self;

        let token_program_id = mint.owner();

        // Do any of these mint extensions exist? If so, need to allocate enough space for the
        // token account counterparts.
        let (has_transfer_fee, has_non_transferable, has_transfer_hook) = {
            let mint_data = mint.try_borrow_data()?;
            let mint_state = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;

            (
                mint_state.get_extension::<TransferFeeConfig>().is_ok(),
                mint_state.get_extension::<NonTransferable>().is_ok(),
                mint_state.get_extension::<TransferHook>().is_ok(),
            )
        };

        let token_account =
            if has_transfer_fee || has_non_transferable || has_transfer_hook || immutable_owner {
                if token_program_id != &spl_token_2022::ID {
                    return Err(super::ERROR_EXTENSIONS_UNSUPPORTED.into());
                }

                // Add to this depending on which extensions exist on the mint.
                let mut total_space = super::BASE_WITH_EXTENSIONS_LEN;

                if has_transfer_fee {
                    total_space += {
                        EMPTY_EXTENSION_LEN // type + length
                        + size_of::<u64>() // withheld_amount
                    };
                }
                // Non-transferable accounts have both non-transferable and immutable owner.
                if has_non_transferable {
                    total_space += 2 * EMPTY_EXTENSION_LEN;
                } else if immutable_owner {
                    total_space += EMPTY_EXTENSION_LEN;
                }
                if has_transfer_hook {
                    total_space += {
                        EMPTY_EXTENSION_LEN // type + length
                        + size_of::<bool>() // transferring
                    };
                }

                // Create the token account by assigning it to the token program.
                let token_account = CreateAccount {
                    payer,
                    to: token_account,
                    program_id: token_program_id,
                    space: Some(total_space),
                    lamports: None,
                }
                .try_into_invoke()?;

                if !has_non_transferable && immutable_owner {
                    super::extensions::InitializeImmutableOwner {
                        token_program_id,
                        account: &token_account,
                    }
                    .into_invoke();
                }

                token_account
            } else {
                if !is_any_token_program_id(token_program_id) {
                    return Err(super::ERROR_EXPECTED_TOKEN_PROGRAM.into());
                }

                // Create the token account by assigning it to the token program.
                CreateAccount {
                    payer,
                    to: token_account,
                    program_id: token_program_id,
                    space: Some(spl_token_2022::state::Account::LEN),
                    lamports: None,
                }
                .try_into_invoke()?
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
pub struct InitializeAccount<'a> {
    pub token_program_id: &'a Pubkey,
    pub account: &'a NoStdAccountInfo,
    pub mint: &'a NoStdAccountInfo,
    pub owner: &'a Pubkey,
}

impl<'a> InitializeAccount<'a> {
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
