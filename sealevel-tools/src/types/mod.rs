use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

/// When specified in [CreateAccount], either find the account by its key in the [AccountInfo]
/// slice (can be expensive) or use the provided [AccountInfo].
pub enum InputAccount<'a, 'b> {
    /// Use this key to find the [AccountInfo] as account to be created.
    Key(&'b Pubkey),

    /// Use this [AccountInfo] as the account to be created.
    Info(&'b AccountInfo<'a>),
}

impl<'a, 'b> InputAccount<'a, 'b> {
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

pub struct InputAuthority<'a, 'b, 'c> {
    pub account: InputAccount<'a, 'c>,
    pub signer_seeds: Option<&'c [&'b [u8]]>,
}
