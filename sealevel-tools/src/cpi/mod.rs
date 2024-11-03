//! Utility methods for cross-program invocations.

pub mod system_program;
#[cfg(feature = "token")]
pub mod token_program;

use core::ops::Deref;

use solana_nostd_entrypoint::{AccountInfoC, AccountMetaC, InstructionC, NoStdAccountInfo};
use solana_program::pubkey::Pubkey;

/// Associate signer seeds with an [NoStdAccountInfo]. Signer seeds may be `None` if
/// [NoStdAccountInfo::is_signer] is true.
pub struct CpiAuthority<'a, 'b> {
    pub account: &'b NoStdAccountInfo,
    pub signer_seeds: Option<&'b [&'a [u8]]>,
}

impl<'a, 'b> CpiAuthority<'a, 'b> {
    pub fn borrow(&self) -> CpiAuthority {
        CpiAuthority {
            account: self.account,
            signer_seeds: self.signer_seeds,
        }
    }
}

impl<'a, 'b> Deref for CpiAuthority<'a, 'b> {
    type Target = NoStdAccountInfo;

    fn deref(&self) -> &Self::Target {
        self.account
    }
}

/// Setup to invoke a cross-program instruction.
pub struct CpiPrecursor<'a, const ACCOUNT_LEN: usize, const DATA_LEN: usize> {
    pub program_id: &'a Pubkey,
    pub accounts: [AccountMetaC; ACCOUNT_LEN],
    pub instruction_data: [u8; DATA_LEN],
    pub infos: [AccountInfoC; ACCOUNT_LEN],
}

impl<'a, const ACCOUNT_LEN: usize, const DATA_LEN: usize> CpiPrecursor<'a, ACCOUNT_LEN, DATA_LEN> {
    pub fn invoke_signed_unchecked(&self, signer_seeds: &[&[&[u8]]]) {
        let Self {
            program_id,
            accounts,
            instruction_data,
            infos,
        } = self;

        let instruction = InstructionC {
            program_id: (*program_id),
            accounts: accounts.as_ptr(),
            accounts_len: accounts.len() as u64,
            data: instruction_data.as_ptr(),
            data_len: instruction_data.len() as u64,
        };

        invoke_signed_unchecked(&instruction, infos, signer_seeds);
    }
}

/// Similar to [invoke_signed](solana_program::program::invoke_signed), but performs a lower level
/// call to the runtime and does not try to perform any account borrows.
pub fn invoke_signed_unchecked(
    instruction: &InstructionC,
    infos: &[AccountInfoC],
    seeds: &[&[&[u8]]],
) {
    // Invoke system program
    #[cfg(target_os = "solana")]
    unsafe {
        solana_program::syscalls::sol_invoke_signed_c(
            instruction as *const InstructionC as *const u8,
            infos.as_ptr() as *const u8,
            infos.len() as u64,
            seeds.as_ptr() as *const u8,
            seeds.len() as u64,
        );
    }

    // For clippy
    #[cfg(not(target_os = "solana"))]
    let _ = (instruction, infos, seeds);
}
