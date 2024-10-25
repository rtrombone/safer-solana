use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

/// Arguments for [try_close_account].
pub struct CloseAccount<'a, 'b> {
    pub account: &'b AccountInfo<'a>,
    pub beneficiary: &'b AccountInfo<'a>,
}

/// Close an account by transferring all lamports to the beneficiary account and assigning the
/// account to the System.
///
/// Inspired by <https://github.com/coral-xyz/anchor/blob/v0.30.1/lang/src/common.rs>.
pub fn try_close_account(
    CloseAccount {
        account,
        beneficiary,
    }: CloseAccount,
) -> ProgramResult {
    // Transfer tokens from the account to the sol_destination.
    let mut account_lamports = account.try_borrow_mut_lamports()?;
    let mut beneficiary_lamports = beneficiary.try_borrow_mut_lamports()?;

    **beneficiary_lamports = beneficiary_lamports.saturating_add(**account_lamports);
    **account_lamports = 0;

    account.assign(&solana_program::system_program::ID);
    account.realloc(0, false)
}
