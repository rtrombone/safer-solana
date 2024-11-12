use sealevel_tools::pubkey::Pubkey;

use crate::ID;

pub const MINT_SEED: &[u8] = b"mint";
pub const TOKEN_SEED: &[u8] = b"token";
pub const AUTHORITY_SEED: &[u8] = b"authority";

#[inline(always)]
pub fn find_mint_address() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED], &ID)
}

#[inline(always)]
pub fn find_token_account_address(owner: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TOKEN_SEED, owner.as_ref()], &ID)
}

#[inline(always)]
pub fn find_authority_address() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[AUTHORITY_SEED], &ID)
}
