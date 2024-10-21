use std::ops::Deref;

use solana_program::{
    account_info::AccountInfo,
    bpf_loader_upgradeable::{UpgradeableLoaderState, ID},
    pubkey::Pubkey,
};

use crate::account_info::NextEnumeratedAccountOptions;

use super::{DataAccount, ProcessNextEnumeratedAccount, Program};

pub struct BpfLoaderUpgradeableProgram<'a, 'b>(pub Program<'a, 'b>);

impl<'a, 'b> ProcessNextEnumeratedAccount<'a, 'b> for BpfLoaderUpgradeableProgram<'a, 'b> {
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            key: Some(&ID),
            ..Program::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if account.key == &ID {
            Some(Self(Program(account)))
        } else {
            None
        }
    }
}

impl<'a, 'b> Deref for BpfLoaderUpgradeableProgram<'a, 'b> {
    type Target = Program<'a, 'b>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct UpgradeableProgramData<'a, 'b, const WRITE: bool> {
    pub account: DataAccount<'a, 'b, WRITE>,
    pub data: (
        u64,            // slot
        Option<Pubkey>, // upgrade_authority_address
    ),
}

impl<'a, 'b, const WRITE: bool> UpgradeableProgramData<'a, 'b, WRITE> {
    pub fn slot(&self) -> u64 {
        self.data.0
    }

    pub fn upgrade_authority_address(&self) -> Option<Pubkey> {
        self.data.1
    }
}

impl<'a, 'b, const WRITE: bool> ProcessNextEnumeratedAccount<'a, 'b>
    for UpgradeableProgramData<'a, 'b, WRITE>
{
    const NEXT_ACCOUNT_OPTIONS: NextEnumeratedAccountOptions<'static, 'static> =
        NextEnumeratedAccountOptions {
            owner: Some(&ID),
            ..DataAccount::<'a, 'b, WRITE>::NEXT_ACCOUNT_OPTIONS
        };

    fn checked_new(account: &'b AccountInfo<'a>) -> Option<Self> {
        if account.owner == &ID {
            let account = DataAccount::checked_new(account)?;

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
