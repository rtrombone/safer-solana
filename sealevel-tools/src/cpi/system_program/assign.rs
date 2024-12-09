use crate::{
    account::system::ID,
    cpi::{CpiAuthority, CpiInstruction},
    pubkey::Pubkey,
};

/// Arguments for the assign instruction on the System program, which assigns ownership of a
/// System-owned account to another program.
#[derive(Clone, PartialEq, Eq)]
pub struct Assign<'a, 'b: 'a> {
    pub to: CpiAuthority<'a, 'b>,
    pub owner: &'a Pubkey,
}

impl<'a, 'b: 'a> Assign<'a, 'b> {
    /// Consume arguments to perform CPI call.
    #[inline(always)]
    pub fn into_invoke(self) {
        let Self { to, owner } = self;

        _invoke_assign(&to, owner);
    }
}

#[inline(always)]
pub(super) fn _invoke_assign(to: &CpiAuthority, owner: &Pubkey) {
    let instruction_data = _serialize_instruction_data(owner);

    CpiInstruction {
        program_id: &ID,
        accounts: &[to.to_meta_c_signer()],
        data: &instruction_data,
    }
    .invoke_possibly_signed(&[to.to_info_c()], &[to.signer_seeds]);
}

const IX_DATA_LEN: usize = {
    4 // selector
    + core::mem::size_of::<Pubkey>() // owner
};

#[inline(always)]
fn _serialize_instruction_data(owner: &Pubkey) -> [u8; IX_DATA_LEN] {
    let mut instruction_data = [0; IX_DATA_LEN];

    // Assign selector == 1.
    instruction_data[0] = 1;
    instruction_data[4..36].copy_from_slice(&owner.to_bytes());

    instruction_data
}

#[cfg(test)]
mod test {
    use solana_sdk::system_instruction::SystemInstruction;

    use super::*;

    #[test]
    fn test_serialize_instruction_data() {
        let owner = Pubkey::new_unique();

        let instruction_data = _serialize_instruction_data(&owner);

        assert_eq!(
            bincode::deserialize::<SystemInstruction>(&instruction_data).unwrap(),
            SystemInstruction::Assign { owner }
        );
    }
}
