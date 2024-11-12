use crate::cpi::{CpiAuthority, CpiInstruction};

/// Arguments for the allocate instruction on the System program, which resizes a System-owned
/// account.
pub struct Allocate<'a, 'b: 'a> {
    pub account: CpiAuthority<'a, 'b>,
    pub space: u64,
}

impl<'a, 'b: 'a> Allocate<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self { account, space } = self;

        _invoke_allocate(&account, space);
    }
}

#[inline(always)]
pub(super) fn _invoke_allocate(account: &CpiAuthority, space: u64) {
    let instruction_data = _serialize_instruction_data(space);

    CpiInstruction {
        program_id: &super::ID,
        accounts: &[account.to_meta_c_signer()],
        data: &instruction_data,
    }
    .invoke_possibly_signed(&[account.to_info_c()], &[account.signer_seeds]);
}

const IX_DATA_LEN: usize = 4 // selector
    + core::mem::size_of::<u64>(); // space

#[inline(always)]
fn _serialize_instruction_data(space: u64) -> [u8; IX_DATA_LEN] {
    let mut instruction_data = [0; IX_DATA_LEN];

    // Allocate selector == 8.
    instruction_data[0] = 8;
    instruction_data[4..12].copy_from_slice(&space.to_le_bytes());

    instruction_data
}

#[cfg(test)]
mod test {
    use solana_program::system_instruction::SystemInstruction;

    use super::*;

    #[test]
    fn test_serialize_instruction_data() {
        let space = 69;

        let instruction_data = _serialize_instruction_data(space);

        assert_eq!(
            bincode::deserialize::<SystemInstruction>(&instruction_data).unwrap(),
            SystemInstruction::Allocate { space }
        );
    }
}
