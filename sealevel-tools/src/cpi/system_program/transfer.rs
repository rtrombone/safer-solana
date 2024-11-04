use solana_nostd_entrypoint::NoStdAccountInfo;

use crate::cpi::{CpiAuthority, CpiInstruction};

/// Arguments for the transfer instruction on the System program, which transfers lamports between
/// two accounts.
pub struct Transfer<'a, 'b> {
    pub from: CpiAuthority<'a, 'b>,
    pub to: &'a NoStdAccountInfo,
    pub lamports: u64,
}

impl<'a, 'b> Transfer<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self { from, to, lamports } = self;

        _invoke_transfer(
            &from,
            &CpiAuthority {
                account: to,
                signer_seeds: None,
            },
            lamports,
        );
    }
}

#[inline(always)]
pub(super) fn _invoke_transfer(from: &CpiAuthority, to: &CpiAuthority, lamports: u64) {
    let instruction_data = _serialize_instruction_data(lamports);

    let from_account = from.account;
    let to_account = to.account;

    CpiInstruction {
        program_id: &super::ID,
        accounts: &[from_account.to_meta_c(), to_account.to_meta_c_signer()],
        data: &instruction_data,
    }
    .invoke_possibly_signed(
        &[from_account.to_info_c(), to_account.to_info_c()],
        &[from.signer_seeds, to.signer_seeds],
    );
}

const IX_DATA_LEN: usize = 4 // selector
    + core::mem::size_of::<u64>(); // lamports

#[inline(always)]
fn _serialize_instruction_data(lamports: u64) -> [u8; IX_DATA_LEN] {
    let mut instruction_data = [0; IX_DATA_LEN];

    // Transfer selector == 2.
    instruction_data[0] = 2;
    instruction_data[4..12].copy_from_slice(&lamports.to_le_bytes());

    instruction_data
}

#[cfg(test)]
mod test {
    use solana_program::system_instruction::SystemInstruction;

    use super::*;

    #[test]
    fn test_serialize_instruction_data() {
        let lamports = 69;

        let instruction_data = _serialize_instruction_data(lamports);

        assert_eq!(
            bincode::deserialize::<SystemInstruction>(&instruction_data).unwrap(),
            SystemInstruction::Transfer { lamports }
        );
    }
}
