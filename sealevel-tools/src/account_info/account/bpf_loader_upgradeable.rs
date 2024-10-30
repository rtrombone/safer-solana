use core::ops::Deref;

use solana_nostd_entrypoint::NoStdAccountInfo;
use solana_program::{
    bpf_loader_upgradeable::{UpgradeableLoaderState, ID},
    pubkey::Pubkey,
};

use crate::account_info::NextEnumeratedAccountOptions;

use super::{Account, ProcessNextEnumeratedAccount, Program};

/// Representing the BPF loader upgradeable program.
pub struct BpfLoaderUpgradeableProgram<'a>(pub Program<'a>);

impl<'a> ProcessNextEnumeratedAccount<'a> for BpfLoaderUpgradeableProgram<'a> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: Some(&ID),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if account.key() == &ID {
            Some(Self(Program(account)))
        } else {
            None
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

impl<'a, const WRITE: bool> UpgradeableProgramData<'a, WRITE> {
    /// The slot at which the program was last upgraded.
    pub fn slot(&self) -> u64 {
        self.data.0
    }

    /// The upgrade authority address. If `None`, the program is immutable.
    pub fn upgrade_authority_address(&self) -> Option<Pubkey> {
        self.data.1
    }
}

impl<'a, const WRITE: bool> ProcessNextEnumeratedAccount<'a> for UpgradeableProgramData<'a, WRITE> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            owner: Some(&ID),
            ..Account::<'a, WRITE>::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'a NoStdAccountInfo) -> Option<Self> {
        if account.owner() == &ID {
            let account = Account::checked_new(account)?;

            let data = {
                let data = account.try_borrow_data().ok()?;
                match bincode::deserialize(&data).ok()? {
                    UpgradeableLoaderState::ProgramData {
                        slot,
                        upgrade_authority_address,
                    } => (slot, upgrade_authority_address),
                    _ => return None,
                }
            };

            Some(Self { account, data })
        } else {
            None
        }
    }
}
