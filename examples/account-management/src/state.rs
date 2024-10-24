use borsh::{BorshDeserialize, BorshSerialize};
use sealevel_tools::{
    account::BorshAccountSerde,
    discriminator::{Discriminate, Discriminator},
};

#[derive(Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub struct Thing {
    pub value: u64,
}

impl Discriminate<8> for Thing {
    const DISCRIMINATOR: [u8; 8] = Discriminator::Sha2(b"state::Thing").to_bytes();
}

impl BorshAccountSerde<8> for Thing {}
