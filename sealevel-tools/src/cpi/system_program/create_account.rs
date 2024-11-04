use core::mem::size_of;

use solana_program::{program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar};

use crate::{
    account::AccountSerde,
    account_info::Account,
    cpi::{CpiAuthority, CpiInstruction},
};

/// Arguments to create an account reliably. If the account already has lamports, it will be topped
/// up to the required rent, allocated with the specified amount of space and assigned to the
/// specified program.
///
/// ### Examples
///
/// ```
/// use sealevel_tools::{
///     account_info::{
///         try_next_enumerated_account, NextEnumeratedAccountOptions, Payer, Program,
///         WritableAccount,
///     },
///     cpi::system_program::CreateAccount,
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
///     CreateAccount {
///         payer: payer.as_cpi_authority(),
///         to: new_account.as_cpi_authority(Some(&[b"thing", &[new_thing_bump]])),
///         program_id,
///         space: Some(16),
///         lamports: None,
///     }
///     .try_into_invoke()?;
///
///     Ok(())
/// }
/// ```
///
/// Use [Self::try_invoke_and_serialize] to create a new data account and serialize data to it.
/// ```
/// use borsh::{BorshDeserialize, BorshSerialize};
/// use sealevel_tools::{
///     account::{AccountSerde, BorshAccountSchema},
///     account_info::{
///         try_next_enumerated_account, NextEnumeratedAccountOptions, Payer, Program,
///         WritableAccount,
///     },
///     cpi::system_program::CreateAccount,
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
///     CreateAccount {
///         payer: payer.as_cpi_authority(),
///         to: new_account.as_cpi_authority(Some(&[b"thing", &[new_thing_bump]])),
///         program_id,
///         space: None,
///         lamports: None,
///     }
///     .try_invoke_and_serialize(&thing)?;
///
///     Ok(())
/// }
/// ```
pub struct CreateAccount<'a, 'b> {
    /// The account that will pay for the rent.
    ///
    /// ### Notes
    ///
    /// Pass in [None] for [CpiAuthority::signer_seeds] if the payer is passed in as a signer.
    pub payer: CpiAuthority<'a, 'b>,

    /// The account to be created.
    ///
    /// ### Notes
    ///
    /// Pass in [None] for [CpiAuthority::signer_seeds] if the account is passed in as a random
    /// keypair.
    pub to: CpiAuthority<'a, 'b>,

    /// The program to assign the account to.
    pub program_id: &'b Pubkey,

    /// The space to allocate for the account. If [None], defaults to zero for
    /// [Self::try_into_invoke] and will be determined by [AccountSerde::try_account_space] for
    /// [Self::try_invoke_and_serialize].
    pub space: Option<usize>,

    pub lamports: Option<u64>,
}

impl<'a, 'b> CreateAccount<'a, 'b> {
    /// Try to consume arguments to perform CPI calls.
    #[inline(always)]
    pub fn try_into_invoke(self) -> Result<Account<'a, true>, ProgramError> {
        let Self {
            payer,
            to,
            program_id,
            space,
            lamports,
        } = self;

        let space = space.unwrap_or_default();
        let lamports = lamports.unwrap_or(Rent::get().unwrap().minimum_balance(space));

        let current_lamports = *to.try_borrow_lamports()?;

        if current_lamports == 0 {
            _invoke_create_account(&payer, &to, lamports, space as u64, program_id);
        } else {
            let lamport_diff = lamports.saturating_sub(current_lamports);

            if lamport_diff != 0 {
                // Transfer remaining lamports.
                super::_invoke_transfer(&payer, &to, lamport_diff);
            }

            if space != 0 {
                // Allocate space.
                super::_invoke_allocate(&to, space as u64);
            }

            // Assign to specified program.
            crate::cpi::system_program::_invoke_assign(&to, program_id);
        }

        // We know that this account was writable, so we are safe to instantiate it like this.
        Ok(Account(to.account))
    }

    /// Try to consume arguments to create a new data account and serialize data to it using the
    /// account's implemented [AccountSerde], which includes its discriminator. This method uses
    /// [Self::try_into_invoke] to create the account and then serializes the data to this account.
    ///
    /// The space to allocate for the account. If not specified, the space will be determined by
    /// [AccountSerde::try_account_space].
    #[inline(always)]
    pub fn try_invoke_and_serialize<const DISC_LEN: usize, T: AccountSerde<DISC_LEN>>(
        mut self,
        account_data: &T,
    ) -> Result<Account<'a, true>, ProgramError> {
        let space = &mut self.space;

        if space.is_none() {
            space.replace(account_data.try_account_space()?);
        }

        let account = self.try_into_invoke()?;

        {
            let mut data = account.try_borrow_mut_data()?;
            account_data.try_serialize_data(&mut data)?;
        }

        Ok(account)
    }
}

#[inline(always)]
fn _invoke_create_account(
    from: &CpiAuthority,
    to: &CpiAuthority,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) {
    // Create account selector == 0.
    let instruction_data = _serialize_instruction_data(lamports, space, owner);

    CpiInstruction {
        program_id: &super::ID,
        accounts: &[from.to_meta_c(), to.to_meta_c_signer()],
        data: &instruction_data,
    }
    .invoke_possibly_signed(
        &[from.to_info_c(), to.to_info_c()],
        &[from.signer_seeds, to.signer_seeds],
    );
}

const IX_DATA_LEN: usize = 4 // selector
    + size_of::<u64>() // lamports
    + size_of::<u64>() // space
    + size_of::<Pubkey>(); // owner

#[inline(always)]
fn _serialize_instruction_data(lamports: u64, space: u64, owner: &Pubkey) -> [u8; IX_DATA_LEN] {
    let mut instruction_data = [0; IX_DATA_LEN];
    instruction_data[4..12].copy_from_slice(&lamports.to_le_bytes());
    instruction_data[12..20].copy_from_slice(&space.to_le_bytes());
    instruction_data[20..52].copy_from_slice(&owner.to_bytes());

    instruction_data
}

#[cfg(test)]
mod test {
    use solana_program::system_instruction::SystemInstruction;

    use super::*;

    #[test]
    fn test_serialize_instruction_data() {
        let lamports = 420;
        let space = 69;
        let owner = Pubkey::new_unique();

        let instruction_data = _serialize_instruction_data(lamports, space, &owner);

        assert_eq!(
            bincode::deserialize::<SystemInstruction>(&instruction_data).unwrap(),
            SystemInstruction::CreateAccount {
                lamports,
                space,
                owner,
            }
        );
    }
}
