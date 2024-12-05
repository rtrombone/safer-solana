/// Namespace for Associated Token Account program ID.
pub mod ata {
    crate::declare_id!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
}

/// Namespace for Token Extensions program ID.
pub mod token_extensions {
    pub use spl_token_2022::ID;
}

/// Namespace for (legacy) Token program ID.
pub mod legacy_token {
    crate::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}

use core::ops::{Deref, DerefMut};

use crate::{
    discriminator::Discriminate,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    spl_token_2022::extension::{BaseState, StateWithExtensions},
};

use super::AccountSerde;

/// Wrapper around a type implementing [BaseState] and [Pack].
#[derive(Clone, PartialEq, Eq)]
pub struct StateWithExtensionsBaseSchema<T: BaseState + Pack>(pub T);

impl<T: BaseState + Pack> Discriminate<0> for StateWithExtensionsBaseSchema<T> {
    const DISCRIMINATOR: [u8; 0] = [];
}

impl<T: BaseState + Pack> Deref for StateWithExtensionsBaseSchema<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BaseState + Pack> DerefMut for StateWithExtensionsBaseSchema<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: BaseState + Pack> AccountSerde<0> for StateWithExtensionsBaseSchema<T> {
    #[inline(always)]
    fn try_deserialize_schema(data: &mut &[u8]) -> Result<Self, ProgramError> {
        let state = StateWithExtensions::<T>::unpack(data)?;
        Ok(Self(state.base))
    }

    #[inline(always)]
    fn try_serialize_schema(&self, _buf: &mut [u8]) -> Result<(), ProgramError> {
        Err(ProgramError::AccountAlreadyInitialized)
    }

    #[inline(always)]
    fn try_account_schema_space(&self) -> Result<usize, ProgramError> {
        Err(ProgramError::AccountAlreadyInitialized)
    }
}

/// Seeds to derive the Associated Token Account address.
pub struct AtaSeeds<'a> {
    pub owner: &'a Pubkey,
    pub mint: &'a Pubkey,
    pub token_program_id: &'a Pubkey,
}

impl<'a> AtaSeeds<'a> {
    /// If the program ID is not provided, the official ATA program ID will be used.
    pub fn find_program_address(&self, program_id: Option<&Pubkey>) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                &self.owner.to_bytes(),
                &self.token_program_id.to_bytes(),
                &self.mint.to_bytes(),
            ],
            program_id.unwrap_or(&ata::ID),
        )
    }
}
