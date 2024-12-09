//! Utility methods for fetching Sysvar account data.

use solana_clock::Clock;
use solana_define_syscall::define_syscall;
use solana_rent::Rent;

define_syscall!(fn sol_get_clock_sysvar(addr: *mut u8) -> u64);
define_syscall!(fn sol_get_rent_sysvar(addr: *mut u8) -> u64);

/// Load [Clock] directly from Solana runtime.
///
/// This is the preferred way to load a sysvar. Calling this method does not incur any
/// deserialization overhead, and does not require the sysvar account to be passed to the program.
#[allow(unexpected_cfgs)]
#[inline(always)]
pub fn get_clock() -> Clock {
    #[cfg(target_os = "solana")]
    {
        let mut rent = Clock::default();
        unsafe { sol_get_clock_sysvar(&mut rent as *mut _ as *mut u8) };
        rent
    }

    #[cfg(not(target_os = "solana"))]
    panic!("Cannot get sysvar on non-Solana targets")
}

/// Load [Clock::epoch] directly from Solana runtime.
#[inline(always)]
pub fn get_clock_epoch() -> u64 {
    get_clock().epoch
}

/// Load [Clock::slot] directly from Solana runtime.
#[inline(always)]
pub fn get_clock_slot() -> u64 {
    get_clock().slot
}

/// Load [Clock::unix_timestamp] directly from Solana runtime.
#[inline(always)]
pub fn get_clock_unix_timestamp() -> i64 {
    get_clock().unix_timestamp
}

/// Load [Rent] directly from Solana runtime.
///
/// This is the preferred way to load a sysvar. Calling this method does not incur any
/// deserialization overhead, and does not require the sysvar account to be passed to the program.
#[allow(unexpected_cfgs)]
#[inline(always)]
pub fn get_rent() -> Rent {
    #[cfg(target_os = "solana")]
    {
        let mut rent = Rent::default();
        unsafe { sol_get_rent_sysvar(&mut rent as *mut _ as *mut u8) };
        rent
    }

    #[cfg(not(target_os = "solana"))]
    panic!("Cannot get sysvar on non-Solana targets")
}

/// Calculate minimum balance due for rent-exemption of a given account data size by loading [Rent]
/// directly from Solana runtime.
#[inline(always)]
pub fn get_rent_minimum_balance(size: usize) -> u64 {
    get_rent().minimum_balance(size)
}
