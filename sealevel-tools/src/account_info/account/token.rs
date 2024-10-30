use core::ops::Deref;

use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

use crate::account_info::NextEnumeratedAccountOptions;

use super::{Account, DataAccount, PackAccount, ProcessNextEnumeratedAccount, Program};

pub const TOKEN_PROGRAM_IDS: [&Pubkey; 2] = [&spl_token::ID, &spl_token_2022::ID];

/// Determine whether the given program ID is either SPL Token or SPL Token 2022 program ID.
pub fn is_any_token_program_id(program_id: &Pubkey) -> bool {
    TOKEN_PROGRAM_IDS.iter().any(|&id| id == program_id)
}

/// Wrapper for [Program] for either SPL Token or SPL Token 2022 program.
pub struct AnyTokenProgram<'a>(pub(crate) Program<'a>);

impl<'a> ProcessNextEnumeratedAccount<'a> for AnyTokenProgram<'a> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            any_of_keys: Some(&TOKEN_PROGRAM_IDS),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if is_any_token_program_id(account.key()) {
            Some(Self(Program(account)))
        } else {
            None
        }
    }
}

impl<'a> Deref for AnyTokenProgram<'a> {
    type Target = Program<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [Account] that deserializes data with [Pack] for either SPL Token or SPL Token
/// 2022 program accounts.
pub struct AnyTokenProgramData<'a, const WRITE: bool, T: Pack + IsInitialized>(
    pub PackAccount<'a, WRITE, T>,
);

pub type MintAccount<'a, const WRITE: bool> =
    AnyTokenProgramData<'a, WRITE, spl_token_2022::state::Mint>;
pub type MintReadOnlyAccount<'a> = MintAccount<'a, false>;
pub type MintWritableAccount<'a> = MintAccount<'a, true>;

pub type TokenAccount<'a, const WRITE: bool> =
    AnyTokenProgramData<'a, WRITE, spl_token_2022::state::Account>;
pub type TokenReadOnlyAccount<'a> = TokenAccount<'a, false>;
pub type TokenWritableAccount<'a> = TokenAccount<'a, true>;

impl<'a, const WRITE: bool, T: Pack + IsInitialized> ProcessNextEnumeratedAccount<'a>
    for AnyTokenProgramData<'a, WRITE, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            any_of_owners: Some(&TOKEN_PROGRAM_IDS),
            ..Account::<'a, WRITE>::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if is_any_token_program_id(account.owner()) {
            DataAccount::checked_new(account).map(Self)
        } else {
            None
        }
    }
}

impl<'a, const WRITE: bool, T: Pack + IsInitialized> Deref for AnyTokenProgramData<'a, WRITE, T> {
    type Target = PackAccount<'a, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [Program] for the SPL Token program.
pub struct TokenProgram<'a>(pub Program<'a>);

impl<'a> ProcessNextEnumeratedAccount<'a> for TokenProgram<'a> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: Some(&spl_token::ID),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if account.key() == &spl_token::ID {
            Some(Self(Program(account)))
        } else {
            None
        }
    }
}

impl<'a> Deref for TokenProgram<'a> {
    type Target = Program<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [Account] that deserializes data with [Pack] for the SPL Token program.
pub struct LegacyTokenProgramData<'a, const WRITE: bool, T: Pack + IsInitialized>(
    pub PackAccount<'a, WRITE, T>,
);

pub type LegacyMintAccount<'a, const WRITE: bool> =
    LegacyTokenProgramData<'a, WRITE, spl_token_2022::state::Mint>;
pub type LegacyMintReadOnlyAccount<'a> = LegacyMintAccount<'a, false>;
pub type LegacyMintWritableAccount<'a> = LegacyMintAccount<'a, true>;

pub type LegacyTokenAccount<'a, const WRITE: bool> =
    LegacyTokenProgramData<'a, WRITE, spl_token_2022::state::Account>;
pub type LegacyTokenReadOnlyAccount<'a> = LegacyTokenAccount<'a, false>;
pub type LegacyTokenWritableAccount<'a> = LegacyTokenAccount<'a, true>;

impl<'a, const WRITE: bool, T: Pack + IsInitialized> ProcessNextEnumeratedAccount<'a>
    for LegacyTokenProgramData<'a, WRITE, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            owner: Some(&spl_token::ID),
            ..Account::<'a, WRITE>::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if account.owner() == &spl_token::ID {
            DataAccount::checked_new(account).map(Self)
        } else {
            None
        }
    }
}

impl<'a, const WRITE: bool, T: Pack + IsInitialized> Deref
    for LegacyTokenProgramData<'a, WRITE, T>
{
    type Target = PackAccount<'a, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [Program] for the SPL Token 2022 program.
pub struct TokenExtensionsProgram<'a>(pub Program<'a>);

impl<'a> ProcessNextEnumeratedAccount<'a> for TokenExtensionsProgram<'a> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: Some(&spl_token_2022::ID),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if account.key() == &spl_token_2022::ID {
            Some(Self(Program(account)))
        } else {
            None
        }
    }
}

impl<'a> Deref for TokenExtensionsProgram<'a> {
    type Target = Program<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [Account] that deserializes data with [Pack] for the SPL Token 2022 program.
pub struct TokenExtensionsProgramData<'a, const WRITE: bool, T: Pack + IsInitialized>(
    pub PackAccount<'a, WRITE, T>,
);

pub type ExtensionsMintAccount<'a, const WRITE: bool> =
    TokenExtensionsProgramData<'a, WRITE, spl_token_2022::state::Mint>;
pub type ExtensionsMintReadOnlyAccount<'a> = ExtensionsMintAccount<'a, false>;
pub type ExtensionsMintWritableAccount<'a> = ExtensionsMintAccount<'a, true>;

pub type ExtensionsTokenAccount<'a, const WRITE: bool> =
    TokenExtensionsProgramData<'a, WRITE, spl_token_2022::state::Account>;
pub type ExtensionsTokenReadOnlyAccount<'a> = ExtensionsTokenAccount<'a, false>;
pub type ExtensionsTokenWritableAccount<'a> = ExtensionsTokenAccount<'a, true>;

impl<'a, const WRITE: bool, T: Pack + IsInitialized> ProcessNextEnumeratedAccount<'a>
    for TokenExtensionsProgramData<'a, WRITE, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            owner: Some(&spl_token_2022::ID),
            ..Account::<'a, WRITE>::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if account.owner() == &spl_token_2022::ID {
            PackAccount::checked_new(account).map(Self)
        } else {
            None
        }
    }
}

impl<'a, const WRITE: bool, T: Pack + IsInitialized> Deref
    for TokenExtensionsProgramData<'a, WRITE, T>
{
    type Target = PackAccount<'a, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
