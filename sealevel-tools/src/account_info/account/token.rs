use core::ops::Deref;

use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

use crate::{account_info::NextEnumeratedAccountOptions, error::SealevelToolsError};

use super::{Account, DataAccount, PackAccount, ProcessNextEnumeratedAccount, Program};

pub const TOKEN_PROGRAM_IDS: [&Pubkey; 2] = [&spl_token::ID, &spl_token_2022::ID];

/// Determine whether the given program ID is either SPL Token or SPL Token 2022 program ID.
pub fn is_any_token_program_id(program_id: &Pubkey) -> bool {
    TOKEN_PROGRAM_IDS.iter().any(|&id| id == program_id)
}

/// Wrapper for [Program] for either SPL Token or SPL Token 2022 program.
pub struct AnyTokenProgram<'a>(pub(crate) Program<'a>);

impl<'a> TryFrom<&'a NoStdAccountInfo> for AnyTokenProgram<'a> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if is_any_token_program_id(account.key()) {
            Program::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected legacy SPL Token or Token Extensions program",
            ]))
        }
    }
}

impl<'a> ProcessNextEnumeratedAccount<'a> for AnyTokenProgram<'a> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            any_of_keys: Some(&TOKEN_PROGRAM_IDS),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };
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

impl<'a, const WRITE: bool, T: Pack + IsInitialized> TryFrom<&'a NoStdAccountInfo>
    for AnyTokenProgramData<'a, WRITE, T>
{
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if is_any_token_program_id(account.owner()) {
            DataAccount::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected legacy SPL Token or Token Extensions program account",
            ])
            .into())
        }
    }
}

impl<'a, const WRITE: bool, T: Pack + IsInitialized> ProcessNextEnumeratedAccount<'a>
    for AnyTokenProgramData<'a, WRITE, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            any_of_owners: Some(&TOKEN_PROGRAM_IDS),
            ..Account::<'a, WRITE>::NEXT_ACCOUNT_OPTIONS
        };
}

impl<'a, const WRITE: bool, T: Pack + IsInitialized> Deref for AnyTokenProgramData<'a, WRITE, T> {
    type Target = PackAccount<'a, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [Program] for the SPL Token program.
pub struct TokenProgram<'a>(pub Program<'a>);

impl<'a> TryFrom<&'a NoStdAccountInfo> for TokenProgram<'a> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.key() == &spl_token::ID {
            Ok(Self(Program::try_from(account)?))
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected legacy SPL Token program",
            ]))
        }
    }
}

impl<'a> ProcessNextEnumeratedAccount<'a> for TokenProgram<'a> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: Some(&spl_token::ID),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };
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

impl<'a, const WRITE: bool, T: Pack + IsInitialized> TryFrom<&'a NoStdAccountInfo>
    for LegacyTokenProgramData<'a, WRITE, T>
{
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.owner() == &spl_token::ID {
            DataAccount::try_from(account).map(Self)
        } else {
            Err(
                SealevelToolsError::AccountInfo(&["Expected legacy SPL Token program account"])
                    .into(),
            )
        }
    }
}

impl<'a, const WRITE: bool, T: Pack + IsInitialized> ProcessNextEnumeratedAccount<'a>
    for LegacyTokenProgramData<'a, WRITE, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            owner: Some(&spl_token::ID),
            ..Account::<'a, WRITE>::NEXT_ACCOUNT_OPTIONS
        };
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

impl<'a> TryFrom<&'a NoStdAccountInfo> for TokenExtensionsProgram<'a> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.key() == &spl_token_2022::ID {
            Program::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected SPL Token Extensions program",
            ]))
        }
    }
}

impl<'a> ProcessNextEnumeratedAccount<'a> for TokenExtensionsProgram<'a> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: Some(&spl_token_2022::ID),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };
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

impl<'a, const WRITE: bool, T: Pack + IsInitialized> TryFrom<&'a NoStdAccountInfo>
    for TokenExtensionsProgramData<'a, WRITE, T>
{
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.owner() == &spl_token_2022::ID {
            DataAccount::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected legacy SPL Token Extensions program account",
            ])
            .into())
        }
    }
}

impl<'a, const WRITE: bool, T: Pack + IsInitialized> ProcessNextEnumeratedAccount<'a>
    for TokenExtensionsProgramData<'a, WRITE, T>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            owner: Some(&spl_token_2022::ID),
            ..Account::<'a, WRITE>::NEXT_ACCOUNT_OPTIONS
        };
}

impl<'a, const WRITE: bool, T: Pack + IsInitialized> Deref
    for TokenExtensionsProgramData<'a, WRITE, T>
{
    type Target = PackAccount<'a, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
