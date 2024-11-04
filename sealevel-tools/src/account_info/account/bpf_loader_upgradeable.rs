use core::ops::Deref;

use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{
    bpf_loader_upgradeable::{UpgradeableLoaderState, ID},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::SealevelToolsError;

use super::{Account, Program};

/// Representing the BPF loader upgradeable program.
pub struct BpfLoaderUpgradeableProgram<'a>(pub Program<'a>);

impl<'a> TryFrom<&'a NoStdAccountInfo> for BpfLoaderUpgradeableProgram<'a> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.key() == &ID {
            Ok(Self(Program::try_from(account)?))
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected BPF Loader Upgradeable program",
            ]))
        }
    }
}

impl<'a> Deref for BpfLoaderUpgradeableProgram<'a> {
    type Target = Program<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Representing a program's program data (owned by the BPF Loader Upgradeable program).
pub struct UpgradeableProgramData<'a, const WRITE: bool> {
    pub account: Account<'a, WRITE>,
    pub data: (
        u64,            // slot
        Option<Pubkey>, // upgrade_authority_address
    ),
}

pub type UpgradeableReadOnlyProgramData<'a> = UpgradeableProgramData<'a, false>;
pub type UpgradeableWritableProgramData<'a> = UpgradeableProgramData<'a, true>;

impl<'a, const WRITE: bool> TryFrom<&'a NoStdAccountInfo> for UpgradeableProgramData<'a, WRITE> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        let account = Account::try_from(account)?;

        if account.owner() == &ID {
            let data = {
                let data = account.try_borrow_data()?;
                match bincode::deserialize(&data).map_err(|_| ProgramError::InvalidAccountData)? {
                    UpgradeableLoaderState::ProgramData {
                        slot,
                        upgrade_authority_address,
                    } => (slot, upgrade_authority_address),
                    _ => {
                        return Err(
                            SealevelToolsError::AccountInfo(&["Expected program data"]).into()
                        )
                    }
                }
            };

            Ok(Self { account, data })
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected BPF Loader Upgradeable program account",
            ])
            .into())
        }
    }
}

impl<'a, const WRITE: bool> UpgradeableProgramData<'a, WRITE> {
    /// The slot at which the program was last upgraded.
    pub fn slot(&self) -> u64 {
        self.data.0
    }

    /// The upgrade authority address. If [None], the program is immutable.
    pub fn upgrade_authority_address(&self) -> Option<Pubkey> {
        self.data.1
    }
}
