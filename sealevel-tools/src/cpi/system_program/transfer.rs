use solana_nostd_entrypoint::NoStdAccountInfo;

use crate::cpi::{CpiAuthority, CpiPrecursor};

/// Arguments for [invoke_transfer_unchecked].
pub struct Transfer<'a, 'b> {
    pub from: CpiAuthority<'a, 'b>,
    pub to: &'b NoStdAccountInfo,
    pub lamports: u64,
}

/// Invokes the transfer instruction on the System program, which transfers lamports between two
/// accounts.
pub fn invoke_transfer_unchecked(Transfer { from, to, lamports }: Transfer) {
    _invoke_transfer_unchecked(
        &from,
        &CpiAuthority {
            account: to,
            signer_seeds: None,
        },
        lamports,
    );
}

#[inline(always)]
pub(super) fn _invoke_transfer_unchecked(from: &CpiAuthority, to: &CpiAuthority, lamports: u64) {
    const IX_DATA_LEN: usize = 4 // selector
        + 8; // lamports

    let mut instruction_data = [0; IX_DATA_LEN];

    // Transfer selector == 2.
    instruction_data[0] = 2;
    instruction_data[4..12].copy_from_slice(&lamports.to_le_bytes());

    let from_account = from.account;
    let to_account = to.account;

    super::_invoke_signed_from_to_unchecked(
        CpiPrecursor {
            program_id: &super::ID,
            accounts: [from_account.to_meta_c(), to_account.to_meta_c_signer()],
            instruction_data,
            infos: [from_account.to_info_c(), to_account.to_info_c()],
        },
        from.signer_seeds,
        to.signer_seeds,
    );
}
