//! Utility methods for cross-program invocations.

#[cfg(feature = "alloc")]
mod alloc;
#[cfg(feature = "token")]
pub mod ata_program;
pub mod system_program;
#[cfg(feature = "token")]
pub mod token_program;

#[cfg(feature = "alloc")]
pub use alloc::*;

pub use solana_program::program::{set_return_data, MAX_RETURN_DATA};

#[cfg(feature = "alloc")]
pub use solana_program::program::get_return_data as checked_dynamic_return_data;

use core::ops::Deref;

use crate::{
    entrypoint::{AccountInfoC, AccountMetaC, InstructionC, NoStdAccountInfo, ProgramResult},
    pubkey::Pubkey,
};

/// Associate signer seeds with an [NoStdAccountInfo]. Signer seeds may be [None] if
/// [NoStdAccountInfo::is_signer] is true.
#[derive(Clone, PartialEq, Eq)]
pub struct CpiAuthority<'a, 'b: 'a> {
    pub account: &'b NoStdAccountInfo,
    pub signer_seeds: Option<&'a [&'b [u8]]>,
}

impl<'a, 'b: 'a> Deref for CpiAuthority<'a, 'b> {
    type Target = NoStdAccountInfo;

    fn deref(&self) -> &Self::Target {
        self.account
    }
}

/// Because [CpiAuthority] can have a [None] value for [CpiAuthority::signer_seeds], this method
/// finds seeds that can be unwrapped returns them in a fixed size array. The number of seeds is
/// returned as well so the remaining array elements can be disregarded when its slice is passed
/// into an invoke method.
#[inline(always)]
pub fn unwrap_signers_seeds<'a, 'b: 'a, const NUM_POSSIBLE: usize>(
    possible_signers_seeds: &[Option<&'a [&'b [u8]]>; NUM_POSSIBLE],
) -> ([&'a [&'b [u8]]; NUM_POSSIBLE], usize) {
    let mut signers_seeds = [Default::default(); NUM_POSSIBLE];
    let mut end = 0;

    possible_signers_seeds.iter().for_each(|seeds| {
        if let Some(seeds) = seeds {
            signers_seeds[end] = *seeds;
            end += 1;
        }
    });

    (signers_seeds, end)
}

/// Setup to invoke a cross-program instruction. To avoid using heap memory, it is recommended to
/// pass in references to fixed arrays of accounts and infos.
#[derive(Debug, Clone)]
pub struct CpiInstruction<'a> {
    pub program_id: &'a Pubkey,

    /// These accounts are be generated with [NoStdAccountInfo::to_meta_c].
    pub accounts: &'a [AccountMetaC],
    pub data: &'a [u8],
}

impl<'a> CpiInstruction<'a> {
    /// Invoke the cross-program instruction with the specified account infos and signer seeds.
    #[inline(always)]
    pub fn invoke_signed(&self, infos: &[AccountInfoC], signers_seeds: &[&[&[u8]]]) {
        let Self {
            program_id,
            accounts,
            data,
        } = *self;

        let instruction = InstructionC {
            program_id,
            accounts: accounts.as_ptr(),
            accounts_len: accounts.len() as u64,
            data: data.as_ptr(),
            data_len: data.len() as u64,
        };

        invoke_signed_c(&instruction, infos, signers_seeds);
    }

    /// Invoke the cross-program instruction with the specified account infos and optional signer
    /// seeds (which may come from [CpiAuthority] structs).
    #[inline(always)]
    pub fn invoke_possibly_signed<const NUM_POSSIBLE: usize>(
        &self,
        infos: &[AccountInfoC],
        possible_signers_seeds: &[Option<&[&[u8]]>; NUM_POSSIBLE],
    ) {
        let (signers_seeds, end) = unwrap_signers_seeds(possible_signers_seeds);
        self.invoke_signed(infos, &signers_seeds[..end]);
    }

    /// Invoke the cross-program instruction with the specified account infos and signer seeds. This
    /// method returns data from the CPI call as a fixed size array of bytes.
    #[inline(always)]
    pub fn checked_return_data<const DATA_LEN: usize>(
        &self,
        infos: &[AccountInfoC],
        signers_seeds: &[&[&[u8]]],
    ) -> Option<[u8; DATA_LEN]> {
        self.invoke_signed(infos, signers_seeds);
        checked_return_data::<DATA_LEN>().map(|(_, data)| data)
    }

    /// Invoke the cross-program instruction with the specified account infos and signer seeds. This
    /// method returns data from the CPI call as a vector of bytes.
    #[cfg(feature = "alloc")]
    #[inline(always)]
    pub fn checked_dynamic_return_data(
        &self,
        infos: &[AccountInfoC],
        signers_seeds: &[&[&[u8]]],
    ) -> Option<::alloc::vec::Vec<u8>> {
        self.invoke_signed(infos, signers_seeds);
        checked_dynamic_return_data().map(|(_, data)| data)
    }
}

/// Similar to [invoke_signed](solana_program::program::invoke_signed), but performs a lower level
/// call to the runtime and does not try to perform any account borrows.
#[allow(unexpected_cfgs)]
#[inline(always)]
pub fn invoke_signed_c(
    instruction: &InstructionC,
    infos: &[AccountInfoC],
    signers_seeds: &[&[&[u8]]],
) {
    #[cfg(target_os = "solana")]
    unsafe {
        solana_program::syscalls::sol_invoke_signed_c(
            instruction as *const InstructionC as *const u8,
            infos.as_ptr() as *const u8,
            infos.len() as u64,
            signers_seeds.as_ptr() as *const u8,
            signers_seeds.len() as u64,
        );
    }

    #[cfg(not(target_os = "solana"))]
    let _ = (instruction, infos, signers_seeds);
}

/// Check lamports and data borrows on [NoStdAccountInfo]. If writable, this method checks mutable
/// borrows. Otherwise it checks immutable borrows. These borrows are checked in
/// [solana_program::program::invoke_signed] before CPI is called (and will be executed in
/// [try_invoke_signed]).
#[inline(always)]
pub fn try_check_borrow_account_info(account_info: &NoStdAccountInfo) -> ProgramResult {
    if account_info.is_writable() {
        let _ = account_info.try_borrow_mut_lamports()?;
        let _ = account_info.try_borrow_mut_data()?;
    } else {
        let _ = account_info.try_borrow_lamports()?;
        let _ = account_info.try_borrow_data()?;
    }

    Ok(())
}

/// Get the return data from an invoked program as a fixed array of bytes (maximum size defined by
/// [solana_program::program::MAX_RETURN_DATA]). If the return data's size differs from the
/// specified array size, this method will return [None].
#[allow(unexpected_cfgs)]
pub fn checked_return_data<const DATA_LEN: usize>() -> Option<(Pubkey, [u8; DATA_LEN])> {
    assert!(
        DATA_LEN <= MAX_RETURN_DATA,
        "Return data size exceeds 1,024 bytes"
    );

    #[cfg(target_os = "solana")]
    {
        let mut buf = [0; DATA_LEN];
        let mut program_id = Pubkey::default();

        let size = unsafe {
            solana_program::syscalls::sol_get_return_data(
                buf.as_mut_ptr(),
                buf.len() as u64,
                &mut program_id,
            )
        };

        if size == 0 || size != (DATA_LEN as u64) {
            None
        } else {
            Some((program_id, buf))
        }
    }

    #[cfg(not(target_os = "solana"))]
    None
}
