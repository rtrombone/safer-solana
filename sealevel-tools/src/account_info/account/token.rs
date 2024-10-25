use std::ops::Deref;

use solana_program::{
    account_info::AccountInfo,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

use crate::account_info::NextEnumeratedAccountOptions;

use super::{DataAccount, PackDataAccount, ProcessNextEnumeratedAccount, Program};

pub const TOKEN_PROGRAM_IDS: [&Pubkey; 2] = [&spl_token::ID, &spl_token_2022::ID];

/// Determine whether the given program ID is either SPL Token or SPL Token 2022 program ID.
pub fn is_any_token_program_id(program_id: &Pubkey) -> bool {
    TOKEN_PROGRAM_IDS.iter().any(|&id| id == program_id)
}

/// Wrapper for [Program] for either SPL Token or SPL Token 2022 program.
pub struct AnyTokenProgram<'a, 'b>(pub(crate) Program<'a, 'b>);

impl<'a, 'b> ProcessNextEnumeratedAccount<'a, 'b> for AnyTokenProgram<'a, 'b> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            any_of_keys: Some(&TOKEN_PROGRAM_IDS),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if is_any_token_program_id(account.key) {
            Some(Self(Program(account)))
        } else {
            None
        }
    }
}

impl<'a, 'b> Deref for AnyTokenProgram<'a, 'b> {
    type Target = Program<'a, 'b>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [DataAccount] that deserializes data with [Pack] for either SPL Token or SPL Token
/// 2022 program accounts.
pub struct AnyTokenProgramData<'a, 'b, const WRITE: bool, T: Pack + IsInitialized>(
    pub PackDataAccount<'a, 'b, WRITE, T>,
);

impl<'a, 'b, const WRITE: bool, T: Pack + IsInitialized> ProcessNextEnumeratedAccount<'a, 'b>
    for AnyTokenProgramData<'a, 'b, WRITE, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            any_of_owners: Some(&TOKEN_PROGRAM_IDS),
            ..DataAccount::<'a, 'b, WRITE>::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if is_any_token_program_id(account.owner) {
            PackDataAccount::checked_new(account).map(Self)
        } else {
            None
        }
    }
}

impl<'a, 'b, const WRITE: bool, T: Pack + IsInitialized> Deref
    for AnyTokenProgramData<'a, 'b, WRITE, T>
{
    type Target = PackDataAccount<'a, 'b, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [Program] for the SPL Token program.
pub struct TokenProgram<'a, 'b>(pub Program<'a, 'b>);

impl<'a, 'b> ProcessNextEnumeratedAccount<'a, 'b> for TokenProgram<'a, 'b> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: Some(&spl_token::ID),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if account.key == &spl_token::ID {
            Some(Self(Program(account)))
        } else {
            None
        }
    }
}

impl<'a, 'b> Deref for TokenProgram<'a, 'b> {
    type Target = Program<'a, 'b>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [DataAccount] that deserializes data with [Pack] for the SPL Token program.
pub struct TokenProgramData<'a, 'b, const WRITE: bool, T: Pack + IsInitialized>(
    pub PackDataAccount<'a, 'b, WRITE, T>,
);

impl<'a, 'b, const WRITE: bool, T: Pack + IsInitialized> ProcessNextEnumeratedAccount<'a, 'b>
    for TokenProgramData<'a, 'b, WRITE, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            owner: Some(&spl_token::ID),
            ..DataAccount::<'a, 'b, WRITE>::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if account.owner == &spl_token::ID {
            PackDataAccount::checked_new(account).map(Self)
        } else {
            None
        }
    }
}

impl<'a, 'b, const WRITE: bool, T: Pack + IsInitialized> Deref
    for TokenProgramData<'a, 'b, WRITE, T>
{
    type Target = PackDataAccount<'a, 'b, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [Program] for the SPL Token 2022 program.
pub struct Token2022Program<'a, 'b>(pub Program<'a, 'b>);

impl<'a, 'b> ProcessNextEnumeratedAccount<'a, 'b> for Token2022Program<'a, 'b> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: Some(&spl_token_2022::ID),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if account.key == &spl_token_2022::ID {
            Some(Self(Program(account)))
        } else {
            None
        }
    }
}

impl<'a, 'b> Deref for Token2022Program<'a, 'b> {
    type Target = Program<'a, 'b>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [DataAccount] that deserializes data with [Pack] for the SPL Token 2022 program.
pub struct Token2022ProgramData<'a, 'b, const WRITE: bool, T: Pack + IsInitialized>(
    pub PackDataAccount<'a, 'b, WRITE, T>,
);

impl<'a, 'b, const WRITE: bool, T: Pack + IsInitialized> ProcessNextEnumeratedAccount<'a, 'b>
    for Token2022ProgramData<'a, 'b, WRITE, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            owner: Some(&spl_token_2022::ID),
            ..DataAccount::<'a, 'b, WRITE>::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if account.owner == &spl_token_2022::ID {
            PackDataAccount::checked_new(account).map(Self)
        } else {
            None
        }
    }
}

impl<'a, 'b, const WRITE: bool, T: Pack + IsInitialized> Deref
    for Token2022ProgramData<'a, 'b, WRITE, T>
{
    type Target = PackDataAccount<'a, 'b, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
