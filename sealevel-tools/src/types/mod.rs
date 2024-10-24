use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

/// Used when either [Pubkey] or [AccountInfo] is an input for a method.
///
/// See [CreateAccount](crate::cpi::system_program::CreateAccount) as an example of how it is used.
pub enum InputAccount<'a, 'b> {
    /// Use this key to find the [AccountInfo] as account to be created.
    Key(&'b Pubkey),

    /// Use this [AccountInfo] as the account to be created.
    Info(&'b AccountInfo<'a>),
}

impl<'a, 'b> InputAccount<'a, 'b> {
    /// Get the [Pubkey] reference from the [InputAccount].
    pub fn key(&'b self) -> &'b Pubkey {
        match self {
            InputAccount::Key(key) => key,
            InputAccount::Info(info) => info.key,
        }
    }
}

impl<'a, 'b> From<&'b Pubkey> for InputAccount<'a, 'b> {
    fn from(pubkey: &'b Pubkey) -> Self {
        InputAccount::Key(pubkey)
    }
}

impl<'a, 'b> From<&'b AccountInfo<'a>> for InputAccount<'a, 'b> {
    fn from(info: &'b AccountInfo<'a>) -> Self {
        InputAccount::Info(info)
    }
}

/// Associate signer seeds with an [InputAccount]. Signer seeds may be `None` if
/// [AccountInfo::is_signer] is true.
pub struct InputAuthority<'a, 'b, 'c> {
    pub account: InputAccount<'a, 'c>,
    pub signer_seeds: Option<&'c [&'b [u8]]>,
}
