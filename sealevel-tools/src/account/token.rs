pub mod token_extensions {
    pub use spl_token_2022::ID;
}

pub mod legacy_token {
    crate::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}

use core::ops::{Deref, DerefMut};

use crate::{
    discriminator::Discriminate,
    program_error::ProgramError,
    program_pack::Pack,
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
