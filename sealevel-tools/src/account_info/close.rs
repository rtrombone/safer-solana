use crate::entrypoint::{NoStdAccountInfo, ProgramResult};

/// Close an account by transferring all lamports to the beneficiary account and assigning the
/// account to the System.
///
/// Inspired by <https://github.com/coral-xyz/anchor/blob/v0.30.1/lang/src/common.rs>.
#[inline(always)]
pub fn try_close_account(
    account: &NoStdAccountInfo,
    beneficiary: &NoStdAccountInfo,
) -> ProgramResult {
    // Transfer tokens from the account to the sol_destination.
    let mut account_lamports = account.try_borrow_mut_lamports()?;
    let mut beneficiary_lamports = beneficiary.try_borrow_mut_lamports()?;

    *beneficiary_lamports = beneficiary_lamports.saturating_add(*account_lamports);
    *account_lamports = 0;

    // Assign the account to the System program.
    let owner = account.to_info_c().owner;

    unsafe {
        core::ptr::write_volatile(
            owner as *mut [u8; 32],
            crate::account::system::ID.to_bytes(),
        );
    }

    // Reallocate data to zero length.
    let data_ptr = account
        .try_borrow_mut_data()
        .map(|mut data| data.as_mut_ptr())?;

    unsafe {
        *(data_ptr.offset(-8) as *mut u64) = 0;
    }

    Ok(())
}
