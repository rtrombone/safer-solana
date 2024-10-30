use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_program::ID, sysvar::Sysvar,
};

use crate::{
    account::AccountSerde,
    account_info::Account,
    cpi::{CpiAuthority, CpiPrecursor},
};

/// Arguments for [try_failsafe_create_account].
pub struct FailsafeCreateAccount<'a, 'b> {
    /// The account that will pay for the rent. Either find the account by its key in
    /// [Self::account_infos] (can be expensive) or use the provided [NoStdAccountInfo].
    ///
    /// NOTE: Seeds for the [Self::payer] signer if the payer is a System account managed by the
    /// program. Pass in [None] if the payer is passed in as a signer.
    pub payer: CpiAuthority<'a, 'b>,

    /// The account to be created.  Either find the account by its key in [Self::account_infos] (can
    /// be expensive) or use the provided [NoStdAccountInfo].
    ///
    /// NOTE: Seeds for the [Self::to] signer if the account is a PDA. Pass in [None] if the account
    /// is passed in as a random keypair.
    pub to: CpiAuthority<'a, 'b>,

    /// The space to allocate for the account.
    pub space: u64,

    /// The program to assign the account to.
    pub program_id: &'b Pubkey,
}

/// Create a new account. If the account already has lamports, it will be topped up to the required
/// rent, allocated with the specified amount of space and assigned to the specified program.
///
/// ### Example
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, NextEnumeratedAccountOptions, Account, Program,
///         Signer,
///     },
///     cpi::system_program::{try_failsafe_create_account, FailsafeCreateAccount},
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
///     let (new_thing_addr, new_thing_bump) =
///         Pubkey::find_program_address(&[b"thing"], program_id);
///
///     // Next account must be writable data account matching PDA address.
///     let (_, new_account) = try_next_enumerated_account::<Account<true>>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&new_thing_addr),
///             ..Default::default()
///         },
///     )?;
///
///     try_failsafe_create_account(
///         FailsafeCreateAccount {
///             payer: payer.as_cpi_authority(),
///             to: new_account.as_cpi_authority(Some(&[b"thing", &[new_thing_bump]])),
///             space: 16,
///             program_id,
///         })?;
///
///     Ok(())
/// }
/// ```
#[inline(always)]
pub fn try_failsafe_create_account<'a>(
    FailsafeCreateAccount {
        payer,
        to,
        space,
        program_id,
    }: FailsafeCreateAccount<'_, 'a>,
) -> Result<Account<'a, true>, ProgramError> {
    let rent_required = Rent::get().unwrap().minimum_balance(space as usize);

    let current_lamports = *to.try_borrow_lamports()?;

    if current_lamports == 0 {
        _invoke_create_account_unchecked(&payer, &to, rent_required, space, program_id);
    } else {
        let lamport_diff = rent_required.saturating_sub(current_lamports);

        if lamport_diff != 0 {
            // Transfer remaining lamports.
            _invoke_transfer_unchecked(&payer, &to, lamport_diff);
        }

        if space != 0 {
            // Allocate space.
            _invoke_allocate_unchecked(&to, space);
        }

        // Assign to specified program.
        _invoke_assign_unchecked(&to, program_id);
    }

    // We know that this account was writable, so we are safe to instantiate it like this.
    Ok(Account(to.account))
}

/// Create a new data account and write borsh-serialized data to it. If the account requires a
/// discriminator, it will be serialized before this data.
///
/// ### Example
///
/// ```
/// use borsh::{BorshDeserialize, BorshSerialize};
/// use sealevel_tools::{
///     account::{AccountSerde, BorshAccountSchema},
///     account_info::{
///         try_next_enumerated_account, NextEnumeratedAccountOptions, Payer, Program,
///         WritableAccount,
///     },
///     cpi::system_program::{try_create_serialized_account, FailsafeCreateAccount},
///     discriminator::{Discriminate, Discriminator},
/// };
/// use solana_nostd_entrypoint::NoStdAccountInfo;
/// use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
///
/// #[derive(Debug, BorshDeserialize, BorshSerialize)]
/// pub struct Thing {
///     pub data: u64,
/// }
///
///
/// impl Discriminate<4> for Thing {
///     const DISCRIMINATOR: [u8; 4] = Discriminator::Sha2(b"Thing").to_bytes();
/// }
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
///     let (new_thing_addr, new_thing_bump) =
///         Pubkey::find_program_address(&[b"thing"], program_id);
///
///     // Next account must be writable data account matching PDA address.
///     let (_, new_account) = try_next_enumerated_account::<WritableAccount>(
///         &mut accounts_iter,
///         NextEnumeratedAccountOptions {
///             key: Some(&new_thing_addr),
///             ..Default::default()
///         },
///     )?;
///
///     let thing = BorshAccountSchema(Thing { data: 420 });
///
///     try_create_serialized_account(
///         FailsafeCreateAccount {
///             payer: payer.as_cpi_authority(),
///             to: new_account.as_cpi_authority(Some(&[b"thing", &[new_thing_bump]])),
///             space: thing.try_account_space()?,
///             program_id,
///         },
///         &thing,
///     )?;
///
///     Ok(())
/// }
/// ```
pub fn try_create_serialized_account<'a, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>>(
    args: super::FailsafeCreateAccount<'_, 'a>,
    account_data: &T,
) -> Result<Account<'a, true>, ProgramError> {
    let account = super::try_failsafe_create_account(args)?;

    {
        let mut data = account.try_borrow_mut_data()?;
        account_data.try_serialize_data(&mut data)?;
    }

    Ok(account)
}

pub struct CreateAccount<'a, 'b> {
    pub from: CpiAuthority<'a, 'b>,
    pub to: CpiAuthority<'a, 'b>,
    pub lamports: u64,
    pub space: u64,
    pub owner: &'b Pubkey,
}

pub fn invoke_create_account_unchecked(
    CreateAccount {
        from,
        to,
        lamports,
        space,
        owner,
    }: CreateAccount,
) {
    _invoke_create_account_unchecked(&from, &to, lamports, space, owner);
}

pub struct Assign<'a, 'b> {
    pub to: CpiAuthority<'a, 'b>,
    pub owner: &'b Pubkey,
}

pub fn invoke_assign_unchecked(Assign { to, owner }: Assign) {
    _invoke_assign_unchecked(&to, owner);
}

pub struct Transfer<'a, 'b> {
    pub from: CpiAuthority<'a, 'b>,
    pub to: &'b NoStdAccountInfo,
    pub lamports: u64,
}

pub fn invoke_transfer_unchecked(Transfer { from, to, lamports }: Transfer) {
    _invoke_transfer_unchecked(
        &from,
        &CpiAuthority {
            account: to,
            signer_seeds: None,
        },
        lamports,
    );
}

pub struct Allocate<'a, 'b> {
    pub account: CpiAuthority<'a, 'b>,
    pub space: u64,
}

pub fn invoke_allocate_unchecked(Allocate { account, space }: Allocate) {
    _invoke_allocate_unchecked(&account, space);
}

#[inline(always)]
fn _invoke_signed_from_to_unchecked<const ACCOUNT_LEN: usize, const DATA_LEN: usize>(
    precursor: CpiPrecursor<ACCOUNT_LEN, DATA_LEN>,
    from_signer_seeds: Option<&[&[u8]]>,
    to_signer_seeds: Option<&[&[u8]]>,
) {
    match (from_signer_seeds, to_signer_seeds) {
        (Some(from_signer_seeds), Some(to_signer_seeds)) => {
            precursor.invoke_signed_unchecked(&[from_signer_seeds, to_signer_seeds])
        }
        (None, Some(to_signer_seeds)) => precursor.invoke_signed_unchecked(&[to_signer_seeds]),
        (Some(from_signer_seeds), None) => precursor.invoke_signed_unchecked(&[from_signer_seeds]),
        (None, None) => precursor.invoke_signed_unchecked(&[]),
    };
}

#[inline(always)]
fn _invoke_create_account_unchecked(
    from: &CpiAuthority,
    to: &CpiAuthority,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) {
    const IX_DATA_LEN: usize = 4 // selector
        + 8 // lamports
        + 8 // space
        + 32; // owner

    // Create account selector == 0.
    let mut instruction_data = [0; IX_DATA_LEN];
    instruction_data[4..12].copy_from_slice(&lamports.to_le_bytes());
    instruction_data[12..20].copy_from_slice(&space.to_le_bytes());
    instruction_data[20..52].copy_from_slice(&owner.to_bytes());

    _invoke_signed_from_to_unchecked(
        CpiPrecursor {
            program_id: &ID,
            accounts: [from.to_meta_c(), to.to_meta_c_signer()],
            instruction_data,
            infos: [from.to_info_c(), to.to_info_c()],
        },
        from.signer_seeds,
        to.signer_seeds,
    );
}

#[inline(always)]
fn _invoke_assign_unchecked(to: &CpiAuthority, owner: &Pubkey) {
    const IX_DATA_LEN: usize = 4 // selector
        + 32; // owner

    let mut instruction_data = [0; IX_DATA_LEN];

    // Assign selector == 1.
    instruction_data[0] = 1;
    instruction_data[4..36].copy_from_slice(&owner.to_bytes());

    CpiPrecursor {
        program_id: &ID,
        accounts: [to.to_meta_c_signer()],
        instruction_data,
        infos: [to.to_info_c()],
    }
    .invoke_signed_unchecked(&[to.signer_seeds.unwrap_or_default()]);
}

#[inline(always)]
fn _invoke_transfer_unchecked(from: &CpiAuthority, to: &CpiAuthority, lamports: u64) {
    const IX_DATA_LEN: usize = 4 // selector
        + 8; // lamports

    let mut instruction_data = [0; IX_DATA_LEN];

    // Transfer selector == 2.
    instruction_data[0] = 2;
    instruction_data[4..12].copy_from_slice(&lamports.to_le_bytes());

    let from_account = from.account;
    let to_account = to.account;

    _invoke_signed_from_to_unchecked(
        CpiPrecursor {
            program_id: &ID,
            accounts: [from_account.to_meta_c(), to_account.to_meta_c_signer()],
            instruction_data,
            infos: [from_account.to_info_c(), to_account.to_info_c()],
        },
        from.signer_seeds,
        to.signer_seeds,
    );
}

#[inline(always)]
fn _invoke_allocate_unchecked(account: &CpiAuthority, space: u64) {
    const IX_DATA_LEN: usize = 4 // selector
        + 8; // space

    let mut instruction_data = [0; IX_DATA_LEN];

    // Allocate selector == 8.
    instruction_data[0] = 8;
    instruction_data[4..12].copy_from_slice(&space.to_le_bytes());

    CpiPrecursor {
        program_id: &ID,
        accounts: [account.to_meta_c_signer()],
        instruction_data,
        infos: [account.to_info_c()],
    }
    .invoke_signed_unchecked(&[account.signer_seeds.unwrap_or_default()]);
}
