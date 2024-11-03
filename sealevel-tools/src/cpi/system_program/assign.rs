use solana_program::pubkey::Pubkey;

use crate::cpi::{CpiAuthority, CpiPrecursor};

/// Arguments for [invoke_assign_unchecked].
pub struct Assign<'a, 'b> {
    pub to: CpiAuthority<'a, 'b>,
    pub owner: &'b Pubkey,
}

/// Invokes the assign instruction on the System program, which assigns ownership of a System-owned
/// account to another program.
pub fn invoke_assign_unchecked(Assign { to, owner }: Assign) {
    _invoke_assign_unchecked(&to, owner);
}

#[inline(always)]
pub(super) fn _invoke_assign_unchecked(to: &CpiAuthority, owner: &Pubkey) {
    const IX_DATA_LEN: usize = 4 // selector
        + 32; // owner

    let mut instruction_data = [0; IX_DATA_LEN];

    // Assign selector == 1.
    instruction_data[0] = 1;
    instruction_data[4..36].copy_from_slice(&owner.to_bytes());

    CpiPrecursor {
        program_id: &super::ID,
        accounts: [to.to_meta_c_signer()],
        instruction_data,
        infos: [to.to_info_c()],
    }
    .invoke_signed_unchecked(&[to.signer_seeds.unwrap_or_default()]);
}
