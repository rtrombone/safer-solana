use core::ops::Deref;

use crate::{
    account::{legacy_token, token_extensions, StateWithExtensionsBaseSchema},
    entrypoint::NoStdAccountInfo,
    error::SealevelToolsError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    spl_token_2022::{
        extension::BaseState,
        state::{Account as BaseTokenAccountState, Mint as BaseMintState},
    },
};

use super::{Account, DataAccount, PackAccount, Program};

pub const TOKEN_PROGRAM_IDS: [&Pubkey; 2] = [&legacy_token::ID, &token_extensions::ID];

/// Determine whether the given program ID is either SPL Token or SPL Token Extensions program ID.
#[inline(always)]
pub fn is_any_token_program_id(program_id: &Pubkey) -> bool {
    TOKEN_PROGRAM_IDS.iter().any(|&id| id == program_id)
}

type StateWithExtensionsBaseAccount<'a, const WRITE: bool, T> =
    DataAccount<'a, WRITE, 0, StateWithExtensionsBaseSchema<T>>;

/// Wrapper for [Program] for either SPL Token or SPL Token Extensions program.
#[derive(Clone, PartialEq, Eq)]
pub struct TokenProgram<'a>(pub(crate) Program<'a>);

impl<'a> TryFrom<&'a NoStdAccountInfo> for TokenProgram<'a> {
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

impl<'a> Deref for TokenProgram<'a> {
    type Target = Program<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Account must be owned by either the SPL Token or SPL Token Extensions program.
#[derive(Clone, PartialEq, Eq)]
pub struct TokenProgramAccount<'a, const WRITE: bool>(pub(crate) Account<'a, WRITE>);

/// Read-only account for either SPL Token or SPL Token Extensions program.
pub type ReadonlyTokenProgramAccount<'a> = TokenProgramAccount<'a, false>;

/// Writable account for either SPL Token or SPL Token Extensions program.
pub type WritableTokenProgramAccount<'a> = TokenProgramAccount<'a, true>;

impl<'a, const WRITE: bool> TryFrom<&'a NoStdAccountInfo> for TokenProgramAccount<'a, WRITE> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if is_any_token_program_id(account.owner()) {
            Account::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected legacy SPL Token or Token Extensions program account",
            ]))
        }
    }
}

impl<'a, const WRITE: bool> Deref for TokenProgramAccount<'a, WRITE> {
    type Target = Account<'a, WRITE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [DataAccount] that deserializes data with [BaseState] for either SPL Token or SPL
/// Token Extensions program.
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from either [ReadonlyTokenProgramAccount] or [WritableTokenProgramAccount] instead
/// of using this type. But if all you need is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
#[derive(Clone, PartialEq, Eq)]
pub struct TokenProgramDataAccount<'a, const WRITE: bool, T: BaseState + Pack>(
    pub(crate) StateWithExtensionsBaseAccount<'a, WRITE, T>,
);

/// Mint account for either SPL Token or SPL Token Extensions program. Deserialized data only represents
/// the base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from either [ReadonlyTokenProgramAccount] or [WritableTokenProgramAccount] instead
/// of using this type. But if all you need is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type MintAccount<'a, const WRITE: bool> = TokenProgramDataAccount<'a, WRITE, BaseMintState>;

/// Mint read-only account for either SPL Token or SPL Token Extensions program. Deserialized data
/// only represents the base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from [ReadonlyTokenProgramAccount] instead of using this type. But if all you need
/// is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type ReadonlyMintAccount<'a> = MintAccount<'a, false>;

/// Mint writable account for either SPL Token or SPL Token Extensions program. Deserialized data
/// only represents the base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from [WritableTokenProgramAccount] instead of using this type. But if all you need
/// is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type WritableMintAccount<'a> = MintAccount<'a, true>;

/// Token account for either SPL Token or SPL Token Extensions program. Deserialized data only
/// represents the base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from either [ReadonlyTokenProgramAccount] or [WritableTokenProgramAccount] instead
/// of using this type. But if all you need is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type TokenAccount<'a, const WRITE: bool> =
    TokenProgramDataAccount<'a, WRITE, BaseTokenAccountState>;

/// Token read-only account for either SPL Token or SPL Token Extensions program. Deserialized data
/// only represents the base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from [ReadonlyTokenProgramAccount] instead of using this type. But if all you need
/// is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type ReadonlyTokenAccount<'a> = TokenAccount<'a, false>;

/// Token writable account for either SPL Token or SPL Token Extensions program. Deserialized data
/// only represents the base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from [WritableTokenProgramAccount] instead of using this type. But if all you need
/// is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type WritableTokenAccount<'a> = TokenAccount<'a, true>;

impl<'a, const WRITE: bool, T: BaseState + Pack> TryFrom<Account<'a, WRITE>>
    for TokenProgramDataAccount<'a, WRITE, T>
{
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: Account<'a, WRITE>) -> Result<Self, Self::Error> {
        if is_any_token_program_id(account.owner()) {
            DataAccount::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected legacy SPL Token or Token Extensions program account",
            ]))
        }
    }
}

impl<'a, const WRITE: bool, T: BaseState + Pack> TryFrom<&'a NoStdAccountInfo>
    for TokenProgramDataAccount<'a, WRITE, T>
{
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        Account::try_from(account).and_then(TryFrom::try_from)
    }
}

impl<'a, const WRITE: bool, T: BaseState + Pack> Deref for TokenProgramDataAccount<'a, WRITE, T> {
    type Target = StateWithExtensionsBaseAccount<'a, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [Program] for the SPL Token program.
#[derive(Clone, PartialEq, Eq)]
pub struct LegacyTokenProgram<'a>(pub(crate) Program<'a>);

impl<'a> TryFrom<&'a NoStdAccountInfo> for LegacyTokenProgram<'a> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.key() == &legacy_token::ID {
            Ok(Self(Program::try_from(account)?))
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected legacy SPL Token program",
            ]))
        }
    }
}

impl<'a> Deref for LegacyTokenProgram<'a> {
    type Target = Program<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Account must be owned by the SPL Token program.
#[derive(Clone, PartialEq, Eq)]
pub struct LegacyTokenProgramAccount<'a, const WRITE: bool>(pub(crate) Account<'a, WRITE>);

/// Read-only account for the SPL Token program.
pub type ReadonlyLegacyTokenProgramAccount<'a> = LegacyTokenProgramAccount<'a, false>;

/// Writable account for the SPL Token program.
pub type WritableLegacyTokenProgramAccount<'a> = LegacyTokenProgramAccount<'a, true>;

impl<'a, const WRITE: bool> TryFrom<&'a NoStdAccountInfo> for LegacyTokenProgramAccount<'a, WRITE> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.owner() == &legacy_token::ID {
            Account::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected legacy SPL Token program account",
            ]))
        }
    }
}

impl<'a, const WRITE: bool> Deref for LegacyTokenProgramAccount<'a, WRITE> {
    type Target = Account<'a, WRITE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [DataAccount] that deserializes data with [Pack] for the SPL Token program.
#[derive(Clone, PartialEq, Eq)]
pub struct LegacyTokenProgramDataAccount<'a, const WRITE: bool, T: Pack + IsInitialized>(
    pub(crate) PackAccount<'a, WRITE, T>,
);

/// Mint account for the SPL Token program.
pub type LegacyMintAccount<'a, const WRITE: bool> =
    LegacyTokenProgramDataAccount<'a, WRITE, BaseMintState>;

/// Read-only mint account for the SPL Token program.
pub type ReadonlyLegacyMintAccount<'a> = LegacyMintAccount<'a, false>;

/// Writable mint account for the SPL Token program.
pub type WritableLegacyMintAccount<'a> = LegacyMintAccount<'a, true>;

/// Token account for the SPL Token program.
pub type LegacyTokenAccount<'a, const WRITE: bool> =
    LegacyTokenProgramDataAccount<'a, WRITE, BaseTokenAccountState>;

/// Read-only token account for the SPL Token program.
pub type ReadonlyLegacyTokenAccount<'a> = LegacyTokenAccount<'a, false>;

/// Writable token account for the SPL Token program.
pub type WritableLegacyTokenAccount<'a> = LegacyTokenAccount<'a, true>;

impl<'a, const WRITE: bool, T: Pack + IsInitialized> TryFrom<Account<'a, WRITE>>
    for LegacyTokenProgramDataAccount<'a, WRITE, T>
{
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: Account<'a, WRITE>) -> Result<Self, Self::Error> {
        if account.owner() == &legacy_token::ID {
            DataAccount::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected legacy SPL Token program account",
            ]))
        }
    }
}

impl<'a, const WRITE: bool, T: Pack + IsInitialized> TryFrom<&'a NoStdAccountInfo>
    for LegacyTokenProgramDataAccount<'a, WRITE, T>
{
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        Account::try_from(account).and_then(TryFrom::try_from)
    }
}

impl<'a, const WRITE: bool, T: Pack + IsInitialized> Deref
    for LegacyTokenProgramDataAccount<'a, WRITE, T>
{
    type Target = PackAccount<'a, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [Program] for the SPL Token Extensions program.
#[derive(Clone, PartialEq, Eq)]
pub struct TokenExtensionsProgram<'a>(pub(crate) Program<'a>);

impl<'a> TryFrom<&'a NoStdAccountInfo> for TokenExtensionsProgram<'a> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.key() == &token_extensions::ID {
            Program::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected SPL Token Extensions program",
            ]))
        }
    }
}

impl<'a> Deref for TokenExtensionsProgram<'a> {
    type Target = Program<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Account must be owned by the SPL Token Extensions program.
#[derive(Clone, PartialEq, Eq)]
pub struct TokenExtensionsProgramAccount<'a, const WRITE: bool>(pub(crate) Account<'a, WRITE>);

/// Read-only account for the SPL Token Extensions program.
pub type ReadonlyTokenExtensionsProgramAccount<'a> = TokenExtensionsProgramAccount<'a, false>;

/// Writable account for the SPL Token Extensions program.
pub type WritableTokenExtensionsProgramAccount<'a> = TokenExtensionsProgramAccount<'a, true>;

impl<'a, const WRITE: bool> TryFrom<&'a NoStdAccountInfo>
    for TokenExtensionsProgramAccount<'a, WRITE>
{
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.owner() == &token_extensions::ID {
            Account::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected SPL Token Extensions program account",
            ]))
        }
    }
}

impl<'a, const WRITE: bool> Deref for TokenExtensionsProgramAccount<'a, WRITE> {
    type Target = Account<'a, WRITE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for [DataAccount] that deserializes data with [BaseState] for either SPL Token or SPL
/// Token Extensions program.
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from either [ReadonlyTokenProgramAccount] or [WritableTokenProgramAccount] instead
/// of using this type. But if all you need is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
#[derive(Clone, PartialEq, Eq)]
pub struct TokenExtensionsProgramDataAccount<'a, const WRITE: bool, T: BaseState + Pack>(
    pub(crate) StateWithExtensionsBaseAccount<'a, WRITE, T>,
);

/// Mint account for for the SPL Token Extensions program. Deserialized data only represents the
/// base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from either [ReadonlyTokenExtensionsProgramAccount] or
/// [WritableTokenExtensionsProgramAccount] instead of using this type. But if all you need is the
/// base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type ExtensionsBaseMintAccount<'a, const WRITE: bool> =
    TokenExtensionsProgramDataAccount<'a, WRITE, BaseMintState>;

/// Read-only mint account for the SPL Token Extensions program. Deserialized data only represents
/// the base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from [ReadonlyTokenExtensionsProgramAccount] instead of using this type. But if all
/// you need is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type ReadonlyExtensionsBaseMintAccount<'a> = ExtensionsBaseMintAccount<'a, false>;

/// Writable mint account for the SPL Token Extensions program. Deserialized data only represents
/// the base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from [WritableTokenExtensionsProgramAccount] instead of using this type. But if
/// all you need is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type WritableExtensionsBaseMintAccount<'a> = ExtensionsBaseMintAccount<'a, true>;

/// Token account for for the SPL Token Extensions program. Deserialized data only represents the
/// base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from either [ReadonlyTokenExtensionsProgramAccount] or
/// [WritableTokenExtensionsProgramAccount] instead of using this type. But if all you need is the
/// base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type ExtensionsBaseTokenAccount<'a, const WRITE: bool> =
    TokenExtensionsProgramDataAccount<'a, WRITE, BaseTokenAccountState>;

/// Read-only token account for the SPL Token Extensions program. Deserialized data only represents
/// the base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from [ReadonlyTokenExtensionsProgramAccount] instead of using this type. But if all
/// you need is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type ReadonlyExtensionsBaseTokenAccount<'a> = ExtensionsBaseTokenAccount<'a, false>;

/// Writable token account for the SPL Token Extensions program. Deserialized data only represents
/// the base state (no extensions).
///
/// ### Notes
///
/// It is recommended to use either [StateWithExtensions] or [PodStateWithExtensions] by borrowing
/// account data from [WritableTokenExtensionsProgramAccount] instead of using this type. But if
/// all you need is the base state, this type is sufficient.
///
/// [PodStateWithExtensions]: spl_token_2022::extension::PodStateWithExtensions
/// [StateWithExtensions]: spl_token_2022::extension::StateWithExtensions
pub type WritableExtensionsBaseTokenAccount<'a> = ExtensionsBaseTokenAccount<'a, true>;

impl<'a, const WRITE: bool, T: BaseState + Pack> TryFrom<Account<'a, WRITE>>
    for TokenExtensionsProgramDataAccount<'a, WRITE, T>
{
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: Account<'a, WRITE>) -> Result<Self, Self::Error> {
        if account.owner() == &token_extensions::ID {
            DataAccount::try_from(account).map(Self)
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected legacy SPL Token program account",
            ]))
        }
    }
}

impl<'a, const WRITE: bool, T: BaseState + Pack> TryFrom<&'a NoStdAccountInfo>
    for TokenExtensionsProgramDataAccount<'a, WRITE, T>
{
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        Account::try_from(account).and_then(TryFrom::try_from)
    }
}

impl<'a, const WRITE: bool, T: BaseState + Pack> Deref
    for TokenExtensionsProgramDataAccount<'a, WRITE, T>
{
    type Target = StateWithExtensionsBaseAccount<'a, WRITE, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
