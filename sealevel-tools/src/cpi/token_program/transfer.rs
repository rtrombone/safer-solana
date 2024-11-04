#[cfg(feature = "alloc")]
use alloc::vec;

use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

use crate::cpi::{CpiAuthority, CpiInstruction};

/// If "alloc" feature is disabled, only this maximum number of additional accounts can be passed
/// into [Transfer] (panics) and [TransferChecked] (returns [Err]).
pub const MAX_ADDITIONAL_ACCOUNTS_NOALLOC: usize = 12;

/// Arguments for the transfer instruction on the specified Token program, which moves a specified
/// amount of tokens from the source to destination token account.
///
/// ### Notes
///
/// When [TransferChecked] is used, the transfer checked instruction is invoked. Otherwise the
/// old transfer instruction (which is marked as deprecated) will be used.
///
/// Also, do not be fooled by the suffix of this function (because all of the invoke methods are
/// suffixed with "unchecked" to indicate a distinction between this library's way of performing CPI
/// calls vs the way found in [solana_program::program::invoke_signed]).
///
/// There is another method called [TransferChecked], which specifically handles invoking the
/// transfer checked instruction. It will perform a length check on the additional accounts if the
/// "alloc" feature is disabled (and will error out if it exceeds
/// [MAX_ADDITIONAL_ACCOUNTS_NOALLOC]).
///
/// ### Examples
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, Authority, NextEnumeratedAccountOptions,
///         TokenProgramWritableAccount, WritableAccount,
///     },
///     cpi::token_program as token_program_cpi,
/// };
/// use solana_nostd_entrypoint::NoStdAccountInfo;
/// use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
///
/// solana_program::declare_id!("Examp1eTokenManagement1111111111111111111111");
///
/// pub fn transfer(accounts: &[NoStdAccountInfo], amount: u64) -> ProgramResult {
///     let mut accounts_iter = accounts.iter().enumerate();
///
///     // First account is the source token account. We don't care to deserialize the token
///     // account.
///     let (_, source_account) = try_next_enumerated_account::<TokenProgramWritableAccount>(
///         &mut accounts_iter,
///         Default::default(),
///     )?;
///
///     let token_program_id = source_account.owner();
///
///     // Second account is the destination token account. No need to check whether this account
///     // belongs to a Token program because we enforce the Token program ID from the source
///     // account.
///     let (_, destination_account) =
///         try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;
///
///     // Third account is the authority, which should have been delegated by the owner of the
///     // source account.
///     let (_, authority) =
///         try_next_enumerated_account::<Authority>(&mut accounts_iter, Default::default())?;
///
///     token_program_cpi::Transfer {
///         token_program_id,
///         source: &source_account,
///         destination: &destination_account,
///         authority: authority.as_cpi_authority(),
///         amount,
///         checked: None,
///     }
///     .into_invoke();
///
///     Ok(())
/// }
/// ```
///
/// Use [UseTransferChecked] to perform a transfer checked instruction.
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, Authority, NextEnumeratedAccountOptions, ReadonlyAccount,
///         TokenProgramWritableAccount, WritableAccount,
///     },
///     cpi::token_program as token_program_cpi,
/// };
/// use solana_nostd_entrypoint::NoStdAccountInfo;
/// use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
///
/// solana_program::declare_id!("Examp1eTokenManagement1111111111111111111111");
///
/// pub fn transfer_checked(
///     accounts: &[NoStdAccountInfo],
///     amount: u64,
///     decimals: u8
/// ) -> ProgramResult {
///     let mut accounts_iter = accounts.iter().enumerate();
///
///     // First account is the source token account. We don't care to deserialize the token
///     // account.
///     let (_, source_account) = try_next_enumerated_account::<TokenProgramWritableAccount>(
///         &mut accounts_iter,
///         Default::default(),
///     )?;
///
///     let token_program_id = source_account.owner();
///
///     // Second account is the destination token account. No need to check whether this account
///     // belongs to a Token program because we enforce the Token program ID from the source
///     // account.
///     let (_, destination_account) =
///         try_next_enumerated_account::<WritableAccount>(&mut accounts_iter, Default::default())?;
///
///     // Third account is the authority, which should have been delegated by the owner of the
///     // source account.
///     let (_, authority) =
///         try_next_enumerated_account::<Authority>(&mut accounts_iter, Default::default())?;
///
///     // Fourth account is the mint. No need to check whether this account belongs to a Token
///     // program because we enforce the Token program ID from the source account.
///     //
///     // Save the index to pass in remaining accounts.
///     let (index, mint_account) =
///         try_next_enumerated_account::<ReadonlyAccount>(&mut accounts_iter, Default::default())?;
///
///     token_program_cpi::Transfer {
///         token_program_id,
///         source: &source_account,
///         destination: &destination_account,
///         authority: authority.as_cpi_authority(),
///         amount,
///         checked: Some(token_program_cpi::UseTransferChecked {
///             mint: &mint_account,
///             decimals,
///             additional_accounts: Some(&accounts[index..]),
///         }),
///     }
///     .into_invoke();
///
///     Ok(())
/// }
/// ```
pub struct Transfer<'a, 'b> {
    pub token_program_id: &'b Pubkey,
    pub source: &'a NoStdAccountInfo,
    pub destination: &'a NoStdAccountInfo,

    /// Either the owner or delegated authority of the source token account.
    pub authority: CpiAuthority<'a, 'b>,
    pub amount: u64,

    /// If [Some], the transfer checked instruction will be used instead of the deprecated transfer
    /// instruction. See [UseTransferChecked] for more information about its usage.
    pub checked: Option<UseTransferChecked<'a>>,
}

/// Optional arguments for [Transfer], which enables the transfer checked instruction instead of the
/// deprecated transfer instruction.
pub struct UseTransferChecked<'a> {
    pub mint: &'a NoStdAccountInfo,
    pub decimals: u8,

    /// These additional accounts apply to the transfer checked instruction on the Token Extensions
    /// program. Examples of these accounts may include signers of the multisig extension or
    /// accounts needed for transfer hook CPI (which is performed by the Token Extensions program).
    ///
    /// ### Notes
    ///
    /// If the "alloc" feature is disabled, only a max number specified by
    /// [MAX_ADDITIONAL_ACCOUNTS_NOALLOC] can be passed in and will panic if more are passed into
    /// [Transfer].
    ///
    /// If additional accounts happen to be passed in for a transfer checked call to the
    /// legacy Token program, the program will disregard these accounts when performing CPI.
    pub additional_accounts: Option<&'a [NoStdAccountInfo]>,
}

impl<'a, 'b> Transfer<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self {
            token_program_id,
            source,
            destination,
            authority,
            amount,
            checked,
        } = self;

        match checked {
            Some(checked) => _invoke_transfer_checked(
                token_program_id,
                source,
                destination,
                &authority,
                amount,
                checked,
            ),
            None => _invoke_transfer(token_program_id, source, destination, &authority, amount),
        }
    }
}

/// Arguments for the transfer checked instruction on the specified Token program, which moves
/// a specified amount of tokens from the source to destination token account only if the specified
/// decimals agree with the decimals encoded in the mint account. Only the token account's owner or
/// delegated authority can invoke this instruction.
///
/// If the "alloc" feature is disabled, this method will error out if the number of additional
/// accounts exceeds [MAX_ADDITIONAL_ACCOUNTS_NOALLOC]. Otherwise this method should be infallible.
pub struct TransferChecked<'a, 'b> {
    pub token_program_id: &'b Pubkey,
    pub source: &'a NoStdAccountInfo,
    pub mint: &'a NoStdAccountInfo,
    pub destination: &'a NoStdAccountInfo,

    /// Either the owner or delegated authority of the source token account.
    pub authority: CpiAuthority<'a, 'b>,
    pub amount: u64,
    pub decimals: u8,

    /// These additional accounts apply to the transfer checked instruction on the Token Extensions
    /// program. Examples of these accounts may include signers of the multisig extension or
    /// accounts needed for transfer hook CPI (which is performed by the Token Extensions program).
    pub additional_accounts: Option<&'a [NoStdAccountInfo]>,
}

impl<'a, 'b> TransferChecked<'a, 'b> {
    /// Tries to consume arguments to perform CPI call.
    #[inline(always)]
    pub fn try_into_invoke(self) -> ProgramResult {
        let Self {
            token_program_id,
            source,
            mint,
            destination,
            authority,
            amount,
            decimals,
            additional_accounts,
        } = self;

        #[cfg(feature = "alloc")]
        {
            _invoke_transfer_checked(
                token_program_id,
                source,
                destination,
                &authority,
                amount,
                UseTransferChecked {
                    mint,
                    decimals,
                    additional_accounts,
                },
            );

            Ok(())
        }

        #[cfg(not(feature = "alloc"))]
        if additional_accounts
            .is_some_and(|accounts| accounts.len() > MAX_ADDITIONAL_ACCOUNTS_NOALLOC)
        {
            return Err(crate::error::SealevelToolsError::Cpi(&[
                "Additional accounts exceed max allowed",
            ])
            .into());
        } else {
            _invoke_transfer_checked(
                token_program_id,
                source,
                destination,
                &authority,
                amount,
                UseTransferChecked {
                    mint,
                    decimals,
                    additional_accounts,
                },
            );

            Ok(())
        }
    }
}

#[inline(always)]
fn _invoke_transfer(
    token_program_id: &Pubkey,
    source: &NoStdAccountInfo,
    destination: &NoStdAccountInfo,
    authority: &CpiAuthority,
    amount: u64,
) {
    // Transfer selector == 3.
    let instruction_data = super::serialize_amount_instruction_data(3, amount);

    CpiInstruction {
        program_id: token_program_id,
        accounts: &[
            source.to_meta_c(),
            destination.to_meta_c(),
            authority.to_meta_c_signer(),
        ],
        data: &instruction_data,
    }
    .invoke_possibly_signed(
        &[
            source.to_info_c(),
            destination.to_info_c(),
            authority.to_info_c(),
        ],
        &[authority.signer_seeds],
    );
}

#[inline(always)]
fn _invoke_transfer_checked(
    token_program_id: &Pubkey,
    source: &NoStdAccountInfo,
    destination: &NoStdAccountInfo,
    authority: &CpiAuthority,
    amount: u64,
    UseTransferChecked {
        mint,
        decimals,
        additional_accounts,
    }: UseTransferChecked,
) {
    // Transfer checked selector == 12.
    let instruction_data = super::serialize_checked_amount_instruction_data(12, amount, decimals);

    match additional_accounts {
        Some(account_infos)
            if token_program_id == &spl_token_2022::ID && !account_infos.is_empty() =>
        {
            _invoke_transfer_checked_with_additional_accounts(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                &instruction_data,
                account_infos,
            )
        }
        Some(_) | None => _invoke_transfer_checked_without_additional_accounts(
            token_program_id,
            source,
            mint,
            destination,
            authority,
            &instruction_data,
        ),
    }
}

#[inline(always)]
fn _invoke_transfer_checked_without_additional_accounts(
    token_program_id: &Pubkey,
    source: &NoStdAccountInfo,
    mint: &NoStdAccountInfo,
    destination: &NoStdAccountInfo,
    authority: &CpiAuthority,
    data: &[u8],
) {
    CpiInstruction {
        program_id: token_program_id,
        accounts: &[
            source.to_meta_c(),
            mint.to_meta_c(),
            destination.to_meta_c(),
            authority.to_meta_c_signer(),
        ],
        data,
    }
    .invoke_possibly_signed(
        &[
            source.to_info_c(),
            mint.to_info_c(),
            destination.to_info_c(),
            authority.to_info_c(),
        ],
        &[authority.signer_seeds],
    );
}

#[cfg(feature = "alloc")]
#[inline(always)]
fn _invoke_transfer_checked_with_additional_accounts(
    token_program_id: &Pubkey,
    source: &NoStdAccountInfo,
    mint: &NoStdAccountInfo,
    destination: &NoStdAccountInfo,
    authority: &CpiAuthority,
    data: &[u8],
    additional_accounts: &[NoStdAccountInfo],
) {
    let mut accounts = vec![
        source.to_meta_c(),
        mint.to_meta_c(),
        destination.to_meta_c(),
        authority.to_meta_c_signer(),
    ];

    let mut infos = vec![
        source.to_info_c(),
        mint.to_info_c(),
        destination.to_info_c(),
        authority.to_info_c(),
    ];

    for account in additional_accounts {
        accounts.push(account.to_meta_c());
        infos.push(account.to_info_c());
    }

    crate::cpi::CpiInstruction {
        program_id: token_program_id,
        accounts: &accounts,
        data,
    }
    .invoke_possibly_signed(&infos, &[authority.signer_seeds]);
}

#[cfg(not(feature = "alloc"))]
use __noalloc::_invoke_transfer_checked_with_additional_accounts;

#[cfg(not(feature = "alloc"))]
mod __noalloc {
    use super::*;

    #[inline(always)]
    pub fn _invoke_transfer_checked_with_additional_accounts(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        match additional_accounts.len() {
            1 => _invoke_transfer_checked_with_additional_accounts_1(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            2 => _invoke_transfer_checked_with_additional_accounts_2(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            3 => _invoke_transfer_checked_with_additional_accounts_3(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            4 => _invoke_transfer_checked_with_additional_accounts_4(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            5 => _invoke_transfer_checked_with_additional_accounts_5(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            6 => _invoke_transfer_checked_with_additional_accounts_6(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            7 => _invoke_transfer_checked_with_additional_accounts_7(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            8 => _invoke_transfer_checked_with_additional_accounts_8(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            9 => _invoke_transfer_checked_with_additional_accounts_9(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            10 => _invoke_transfer_checked_with_additional_accounts_10(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            11 => _invoke_transfer_checked_with_additional_accounts_11(
                token_program_id,
                source,
                mint,
                destination,
                authority,
                data,
                additional_accounts,
            ),
            MAX_ADDITIONAL_ACCOUNTS_NOALLOC => {
                _invoke_transfer_checked_with_additional_accounts_12(
                    token_program_id,
                    source,
                    mint,
                    destination,
                    authority,
                    data,
                    additional_accounts,
                )
            }
            _ => panic!("Too many additional accounts passed in for transfer checked CPI"),
        }
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_1(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_2(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_3(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
                additional_accounts[2].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
                additional_accounts[2].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_4(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
                additional_accounts[2].to_meta_c(),
                additional_accounts[3].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
                additional_accounts[2].to_info_c(),
                additional_accounts[3].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_5(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
                additional_accounts[2].to_meta_c(),
                additional_accounts[3].to_meta_c(),
                additional_accounts[4].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
                additional_accounts[2].to_info_c(),
                additional_accounts[3].to_info_c(),
                additional_accounts[4].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_6(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
                additional_accounts[2].to_meta_c(),
                additional_accounts[3].to_meta_c(),
                additional_accounts[4].to_meta_c(),
                additional_accounts[5].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
                additional_accounts[2].to_info_c(),
                additional_accounts[3].to_info_c(),
                additional_accounts[4].to_info_c(),
                additional_accounts[5].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_7(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
                additional_accounts[2].to_meta_c(),
                additional_accounts[3].to_meta_c(),
                additional_accounts[4].to_meta_c(),
                additional_accounts[5].to_meta_c(),
                additional_accounts[6].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
                additional_accounts[2].to_info_c(),
                additional_accounts[3].to_info_c(),
                additional_accounts[4].to_info_c(),
                additional_accounts[5].to_info_c(),
                additional_accounts[6].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_8(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
                additional_accounts[2].to_meta_c(),
                additional_accounts[3].to_meta_c(),
                additional_accounts[4].to_meta_c(),
                additional_accounts[5].to_meta_c(),
                additional_accounts[6].to_meta_c(),
                additional_accounts[7].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
                additional_accounts[2].to_info_c(),
                additional_accounts[3].to_info_c(),
                additional_accounts[4].to_info_c(),
                additional_accounts[5].to_info_c(),
                additional_accounts[6].to_info_c(),
                additional_accounts[7].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_9(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
                additional_accounts[2].to_meta_c(),
                additional_accounts[3].to_meta_c(),
                additional_accounts[4].to_meta_c(),
                additional_accounts[5].to_meta_c(),
                additional_accounts[6].to_meta_c(),
                additional_accounts[7].to_meta_c(),
                additional_accounts[8].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
                additional_accounts[2].to_info_c(),
                additional_accounts[3].to_info_c(),
                additional_accounts[4].to_info_c(),
                additional_accounts[5].to_info_c(),
                additional_accounts[6].to_info_c(),
                additional_accounts[7].to_info_c(),
                additional_accounts[8].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_10(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
                additional_accounts[2].to_meta_c(),
                additional_accounts[3].to_meta_c(),
                additional_accounts[4].to_meta_c(),
                additional_accounts[5].to_meta_c(),
                additional_accounts[6].to_meta_c(),
                additional_accounts[7].to_meta_c(),
                additional_accounts[8].to_meta_c(),
                additional_accounts[9].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
                additional_accounts[2].to_info_c(),
                additional_accounts[3].to_info_c(),
                additional_accounts[4].to_info_c(),
                additional_accounts[5].to_info_c(),
                additional_accounts[6].to_info_c(),
                additional_accounts[7].to_info_c(),
                additional_accounts[8].to_info_c(),
                additional_accounts[9].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_11(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
                additional_accounts[2].to_meta_c(),
                additional_accounts[3].to_meta_c(),
                additional_accounts[4].to_meta_c(),
                additional_accounts[5].to_meta_c(),
                additional_accounts[6].to_meta_c(),
                additional_accounts[7].to_meta_c(),
                additional_accounts[8].to_meta_c(),
                additional_accounts[9].to_meta_c(),
                additional_accounts[10].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
                additional_accounts[2].to_info_c(),
                additional_accounts[3].to_info_c(),
                additional_accounts[4].to_info_c(),
                additional_accounts[5].to_info_c(),
                additional_accounts[6].to_info_c(),
                additional_accounts[7].to_info_c(),
                additional_accounts[8].to_info_c(),
                additional_accounts[9].to_info_c(),
                additional_accounts[10].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }

    #[inline(always)]
    fn _invoke_transfer_checked_with_additional_accounts_12(
        token_program_id: &Pubkey,
        source: &NoStdAccountInfo,
        mint: &NoStdAccountInfo,
        destination: &NoStdAccountInfo,
        authority: &CpiAuthority,
        data: &[u8],
        additional_accounts: &[NoStdAccountInfo],
    ) {
        CpiInstruction {
            program_id: token_program_id,
            accounts: &[
                source.to_meta_c(),
                mint.to_meta_c(),
                destination.to_meta_c(),
                authority.to_meta_c_signer(),
                additional_accounts[0].to_meta_c(),
                additional_accounts[1].to_meta_c(),
                additional_accounts[2].to_meta_c(),
                additional_accounts[3].to_meta_c(),
                additional_accounts[4].to_meta_c(),
                additional_accounts[5].to_meta_c(),
                additional_accounts[6].to_meta_c(),
                additional_accounts[7].to_meta_c(),
                additional_accounts[8].to_meta_c(),
                additional_accounts[9].to_meta_c(),
                additional_accounts[10].to_meta_c(),
                additional_accounts[11].to_meta_c(),
            ],
            data,
        }
        .invoke_possibly_signed(
            &[
                source.to_info_c(),
                mint.to_info_c(),
                destination.to_info_c(),
                authority.to_info_c(),
                additional_accounts[0].to_info_c(),
                additional_accounts[1].to_info_c(),
                additional_accounts[2].to_info_c(),
                additional_accounts[3].to_info_c(),
                additional_accounts[4].to_info_c(),
                additional_accounts[5].to_info_c(),
                additional_accounts[6].to_info_c(),
                additional_accounts[7].to_info_c(),
                additional_accounts[8].to_info_c(),
                additional_accounts[9].to_info_c(),
                additional_accounts[10].to_info_c(),
                additional_accounts[11].to_info_c(),
            ],
            &[authority.signer_seeds],
        );
    }
}
