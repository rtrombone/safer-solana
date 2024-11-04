use solana_program::{program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar};

use crate::{
    account::AccountSerde,
    account_info::Account,
    cpi::{CpiAuthority, CpiPrecursor},
};

/// Arguments for [try_failsafe_create_account].
pub struct FailsafeCreateAccount<'a, 'b> {
    /// The account that will pay for the rent.
    ///
    /// NOTE: Pass in `None` for [CpiAuthority::signer_seeds] if the payer is passed in as a signer.
    pub payer: CpiAuthority<'a, 'b>,

    /// The account to be created.
    ///
    /// NOTE: Pass in `None` for [CpiAuthority::signer_seeds] if the account is passed in as a
    /// random keypair.
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
            super::_invoke_transfer_unchecked(&payer, &to, lamport_diff);
        }

        if space != 0 {
            // Allocate space.
            super::_invoke_allocate_unchecked(&to, space);
        }

        // Assign to specified program.
        crate::cpi::system_program::_invoke_assign_unchecked(&to, program_id);
    }

    // We know that this account was writable, so we are safe to instantiate it like this.
    Ok(Account(to.account))
}

/// Arguments for [try_create_serialized_account].
pub struct CreateSerializedAccount<'a, 'b> {
    /// The account that will pay for the rent.
    ///
    /// NOTE: Pass in `None` for [CpiAuthority::signer_seeds] if the payer is passed in as a signer.
    pub payer: CpiAuthority<'a, 'b>,

    /// The account to be created.
    ///
    /// NOTE: Pass in `None` for [CpiAuthority::signer_seeds] if the account is passed in as a
    /// random keypair.
    pub to: CpiAuthority<'a, 'b>,

    /// The program to assign the account to.
    pub program_id: &'b Pubkey,

    /// The space to allocate for the account. If not specified, the space will be determined by
    /// [AccountSerde::try_account_space].
    pub space: Option<u64>,
}

/// Create a new data account and serialize data to it using the account's implemented
/// [AccountSerde], which includes its discriminator. This method uses [try_failsafe_create_account]
/// to create the account and then serializes the data to it.
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
///     cpi::system_program::{try_create_serialized_account, CreateSerializedAccount},
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
///         CreateSerializedAccount {
///             payer: payer.as_cpi_authority(),
///             to: new_account.as_cpi_authority(Some(&[b"thing", &[new_thing_bump]])),
///             program_id,
///             space: None,
///         },
///         &thing,
///     )?;
///
///     Ok(())
/// }
/// ```
pub fn try_create_serialized_account<'a, const DISC_LEN: usize, T: AccountSerde<DISC_LEN>>(
    CreateSerializedAccount {
        payer,
        to,
        space,
        program_id,
    }: CreateSerializedAccount<'_, 'a>,
    account_data: &T,
) -> Result<Account<'a, true>, ProgramError> {
    let space = match space {
        Some(space) => space,
        None => account_data.try_account_space()?,
    };

    let account = super::try_failsafe_create_account(FailsafeCreateAccount {
        payer,
        to,
        space,
        program_id,
    })?;

    {
        let mut data = account.try_borrow_mut_data()?;
        account_data.try_serialize_data(&mut data)?;
    }

    Ok(account)
}

/// Arguments for [invoke_create_account_unchecked].
pub struct CreateAccount<'a, 'b> {
    pub from: CpiAuthority<'a, 'b>,
    pub to: CpiAuthority<'a, 'b>,
    pub lamports: u64,
    pub space: u64,
    pub owner: &'b Pubkey,
}

/// Invokes the create account instruction on the System program, which creates a new account owned
/// by a specified program. Only use this instruction if you are certain the account does not have
/// any lamports on it.
///
/// NOTE: It is preferred to use [try_failsafe_create_account] instead of this method because it
/// performs a check to see if the account already has lamports before invoking this instruction
/// (otherwise, it will top up the account to the required rent, allocate and assign the account to
/// the program).
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

    super::_invoke_signed_from_to_unchecked(
        CpiPrecursor {
            program_id: &super::ID,
            accounts: [from.to_meta_c(), to.to_meta_c_signer()],
            instruction_data,
            infos: [from.to_info_c(), to.to_info_c()],
        },
        from.signer_seeds,
        to.signer_seeds,
    );
}
