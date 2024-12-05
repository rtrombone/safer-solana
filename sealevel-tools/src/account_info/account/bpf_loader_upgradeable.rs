use core::ops::Deref;

use solana_program::bpf_loader_upgradeable::{UpgradeableLoaderState, ID};

use crate::{
    entrypoint::NoStdAccountInfo, error::SealevelToolsError, program_error::ProgramError,
    pubkey::Pubkey,
};

use super::{Account, Program};

const SIZE_OF_PROGRAMDATA_METADATA: usize = UpgradeableLoaderState::size_of_programdata_metadata();

/// Representing the BPF loader Upgradeable program.
#[derive(Clone, PartialEq, Eq)]
pub struct BpfLoaderUpgradeableProgram<'a>(pub(crate) Program<'a>);

impl<'a> TryFrom<&'a NoStdAccountInfo> for BpfLoaderUpgradeableProgram<'a> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        if account.key() == &ID {
            Program::try_from(account).map(Self)
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

/// Account representing a program's program data (owned by the BPF Loader Upgradeable program).
#[derive(Clone, PartialEq, Eq)]
pub struct UpgradeableProgramData<'a, const WRITE: bool> {
    pub(crate) account: Account<'a, WRITE>,
    pub data: (
        u64,            // slot
        Option<Pubkey>, // upgrade_authority_address
    ),
}

/// Read-only account representing a program's program data (owned by the BPF Loader Upgradeable
/// program).
pub type ReadonlyUpgradeableProgramData<'a> = UpgradeableProgramData<'a, false>;

/// Writable account representing a program's program data (owned by the BPF Loader Upgradeable
/// program).
pub type WritableUpgradeableProgramData<'a> = UpgradeableProgramData<'a, true>;

impl<'a, const WRITE: bool> TryFrom<&'a NoStdAccountInfo> for UpgradeableProgramData<'a, WRITE> {
    type Error = SealevelToolsError<'static>;

    #[inline(always)]
    fn try_from(account: &'a NoStdAccountInfo) -> Result<Self, Self::Error> {
        let account = Account::try_from(account)?;

        if account.owner() == &ID {
            let data = {
                let account_data = account.try_borrow_data()?;
                try_deserialize_program_data(&account_data)?
            };

            Ok(Self { account, data })
        } else {
            Err(SealevelToolsError::AccountInfo(&[
                "Expected BPF Loader Upgradeable program account",
            ]))
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

impl<'a, const WRITE: bool> Deref for UpgradeableProgramData<'a, WRITE> {
    type Target = Account<'a, WRITE>;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

fn try_deserialize_program_data(
    account_data: &[u8],
) -> Result<(u64, Option<Pubkey>), SealevelToolsError<'static>> {
    if account_data.len() < SIZE_OF_PROGRAMDATA_METADATA {
        Err(SealevelToolsError::PassThrough(
            ProgramError::InvalidAccountData,
        ))
    } else if account_data[..4] != [3, 0, 0, 0] {
        Err(SealevelToolsError::AccountInfo(&["Expected program data"]))
    } else {
        let slot = {
            let mut buf = [0; 8];
            buf.copy_from_slice(&account_data[4..12]);
            u64::from_le_bytes(buf)
        };

        let upgrade_authority_address = if account_data[12] == 1 {
            let mut buf = [0; 32];
            buf.copy_from_slice(&account_data[13..45]);
            Some(buf.into())
        } else {
            None
        };

        Ok((slot, upgrade_authority_address))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_try_deserialize_program_data() {
        let slot = 42;
        let upgrade_authority_address = Some(Pubkey::new_unique());

        let account_data = bincode::serialize(&UpgradeableLoaderState::ProgramData {
            slot,
            upgrade_authority_address,
        })
        .unwrap();

        assert_eq!(
            try_deserialize_program_data(&account_data).unwrap(),
            (slot, upgrade_authority_address)
        );
    }
}
