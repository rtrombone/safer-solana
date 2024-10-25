//! Utility methods for cross-program invocations.

pub mod system_program;
#[cfg(feature = "token")]
pub mod token_program;

use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

/// Used when either [Pubkey] or [AccountInfo] is an input for a CPI call.
///
/// See [CreateAccount](crate::cpi::system_program::CreateAccount) as an example of how it is used.
#[derive(Debug)]
pub enum CpiAccount<'a, 'b> {
    /// Use this key to find the [AccountInfo] as account to be created.
    Key(&'b Pubkey),

    /// Use this [AccountInfo] as the account to be created.
    Info(&'b AccountInfo<'a>),
}

impl<'a, 'b> CpiAccount<'a, 'b> {
    /// Get the [Pubkey] reference from the [CpiAccount].
    pub fn key(&'b self) -> &'b Pubkey {
        match self {
            CpiAccount::Key(key) => key,
            CpiAccount::Info(info) => info.key,
        }
    }
}

impl<'a, 'b> From<&'b Pubkey> for CpiAccount<'a, 'b> {
    fn from(pubkey: &'b Pubkey) -> Self {
        CpiAccount::Key(pubkey)
    }
}

impl<'a, 'b> From<&'b AccountInfo<'a>> for CpiAccount<'a, 'b> {
    fn from(info: &'b AccountInfo<'a>) -> Self {
        CpiAccount::Info(info)
    }
}

/// Associate signer seeds with an [CpiAccount]. Signer seeds may be `None` if
/// [AccountInfo::is_signer] is true.
#[derive(Debug)]
pub struct CpiAuthority<'a, 'b, 'c> {
    pub account: CpiAccount<'a, 'c>,
    pub signer_seeds: Option<&'c [&'b [u8]]>,
}
