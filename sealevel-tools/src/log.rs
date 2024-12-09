//! Logging utilities forked from [solana_program]'s [log] submodule.
//!
//! Logging is the main mechanism for getting debugging information out of
//! running Solana programs, and there are several functions available for doing
//! so efficiently, depending on the type of data being logged.
//!
//! The most common way to emit logs is through the [msg!] macro, which logs
//! simple strings, as well as [formatted strings].
//!
//! Logs can be viewed in multiple ways:
//!
//! - The `solana logs` command displays logs for all transactions executed on a
//!   network. Note though that transactions that fail during pre-flight
//!   simulation are not displayed here.
//! - When submitting transactions via [RpcClient], if Rust's own logging is
//!   active then the `solana_rpc_client` crate logs at the "debug" level any logs
//!   for transactions that failed during simulation. If using [env_logger]
//!   these logs can be activated by setting `RUST_LOG=solana_rpc_client=debug`.
//! - Logs can be retrieved from a finalized transaction by calling
//!   [RpcClient::get_transaction].
//! - Block explorers may display logs.
//!
//! [RpcClient]: https://docs.rs/solana-rpc-client/latest/solana_rpc_client/rpc_client/struct.RpcClient.html
//! [env_logger]: https://docs.rs/env_logger
//! [RpcClient::get_transaction]: https://docs.rs/solana-rpc-client/latest/solana_rpc_client/rpc_client/struct.RpcClient.html#method.get_transaction
//!
//! While most logging functions are defined in this module, a [Pubkey] can
//! also be efficiently logged with the [Pubkey::log] function.
//!
//! [Pubkey]: crate::pubkey::Pubkey
//! [Pubkey::log]: crate::pubkey::Pubkey::log
//! [formatted strings]: https://doc.rust-lang.org/std/fmt/
//! [log]: https://docs.rs/solana-program/latest/solana_program/log/index.html
//! [solana_program]: https://docs.rs/solana-program/

use crate::account_info::NoStdAccountInfo;
pub use solana_msg::{msg, sol_log};

/// Print 64-bit values represented as hexadecimal to the log.
#[allow(unexpected_cfgs)]
#[inline(always)]
pub fn sol_log_64(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
    #[cfg(target_os = "solana")]
    unsafe {
        solana_msg::syscalls::sol_log_64_(arg1, arg2, arg3, arg4, arg5);
    }

    #[cfg(not(target_os = "solana"))]
    {
        let _ = (arg1, arg2, arg3, arg4, arg5);
        sol_log("sol_log_64() not available");
    }
}

/// Print some slices as base64.
#[allow(unexpected_cfgs)]
#[inline(always)]
pub fn sol_log_data(data: &[&[u8]]) {
    #[cfg(target_os = "solana")]
    unsafe {
        solana_msg::syscalls::sol_log_data(data as *const _ as *const u8, data.len() as u64)
    };

    #[cfg(not(target_os = "solana"))]
    {
        let _ = data;
        sol_log("sol_log_data() not available");
    }
}

/// Print the hexadecimal representation of a slice.
#[inline(always)]
pub fn sol_log_slice(slice: &[u8]) {
    for (i, s) in slice.iter().enumerate() {
        sol_log_64(0, 0, 0, i as u64, *s as u64);
    }
}

/// Print the hexadecimal representation of the program's input parameters.
///
/// - `accounts` - A slice of [NoStdAccountInfo].
/// - `data` - The instruction data.
#[inline(always)]
pub fn sol_log_params(accounts: &[NoStdAccountInfo], data: &[u8]) {
    for (i, account) in accounts.iter().enumerate() {
        sol_log("NoStdAccountInfo");
        sol_log_64(0, 0, 0, 0, i as u64);
        sol_log("- Is signer");
        sol_log_64(0, 0, 0, 0, account.is_signer() as u64);
        sol_log("- Key");
        account.key().log();
        sol_log("- Lamports");
        let lamports = unsafe { *account.unchecked_borrow_lamports() };
        sol_log_64(0, 0, 0, 0, lamports);
        sol_log("- Account data length");
        sol_log_64(0, 0, 0, 0, account.data_len() as u64);
        sol_log("- Owner");
        account.owner().log();
    }
    sol_log("Instruction data");
    sol_log_slice(data);
}

/// Print the remaining compute units available to the program.
#[allow(unexpected_cfgs)]
#[inline(always)]
pub fn sol_log_compute_units() {
    #[cfg(target_os = "solana")]
    unsafe {
        solana_msg::syscalls::sol_log_compute_units_();
    }

    #[cfg(not(target_os = "solana"))]
    sol_log("sol_log_compute_units() not available");
}
