use borsh::{BorshDeserialize, BorshSerialize};
use sealevel_tools::{
    account::BorshAccountSerde,
    discriminator::{Discriminate, Discriminator},
    pda::DeriveAddress,
};
use solana_program::pubkey::Pubkey;

#[derive(Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub struct Thing {
    pub value: u64,
}

impl Thing {
    pub const SEED: &'static [u8] = b"thing";
}

impl Discriminate<8> for Thing {
    const DISCRIMINATOR: [u8; 8] = Discriminator::Sha2(b"state::Thing").to_bytes();
}

impl BorshAccountSerde<8> for Thing {}

impl DeriveAddress for Thing {
    type Seeds<'a> = ();

    fn find_program_address(_seeds: Self::Seeds<'_>) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Thing::SEED], &crate::ID)
    }

    fn create_program_address(_seeds: Self::Seeds<'_>, bump_seed: u8) -> Option<Pubkey> {
        Pubkey::create_program_address(&[Thing::SEED, &[bump_seed]], &crate::ID).ok()
    }
}
