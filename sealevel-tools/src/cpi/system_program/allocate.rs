use crate::cpi::{CpiAuthority, CpiPrecursor};

/// Arguments for [invoke_allocate_unchecked].
pub struct Allocate<'a, 'b> {
    pub account: CpiAuthority<'a, 'b>,
    pub space: u64,
}

/// Invokes the allocate instruction on the System program, which resizes a System-owned account.
pub fn invoke_allocate_unchecked(Allocate { account, space }: Allocate) {
    _invoke_allocate_unchecked(&account, space);
}

#[inline(always)]
pub(super) fn _invoke_allocate_unchecked(account: &CpiAuthority, space: u64) {
    const IX_DATA_LEN: usize = 4 // selector
        + 8; // space

    let mut instruction_data = [0; IX_DATA_LEN];

    // Allocate selector == 8.
    instruction_data[0] = 8;
    instruction_data[4..12].copy_from_slice(&space.to_le_bytes());

    CpiPrecursor {
        program_id: &super::ID,
        accounts: [account.to_meta_c_signer()],
        instruction_data,
        infos: [account.to_info_c()],
    }
    .invoke_signed_unchecked(&[account.signer_seeds.unwrap_or_default()]);
}
