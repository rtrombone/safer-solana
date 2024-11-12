//! CPI for SPL Token Extensions program to manage specific mint and token extensions.
//!
//! ### Notes
//!
//! This module does not have optimized CPI calls for every instruction. For any instruction you
//! need for your program that does not exist here, please use [invoke_signed] with instruction
//! builders found in the [spl_token_2022::instruction] module.
//!
//! [invoke_signed]: crate::cpi::invoke_signed

mod initialize_close_authority;
mod initialize_group_member_pointer;
mod initialize_group_pointer;
mod initialize_immutable_owner;
mod initialize_metadata_pointer;
mod initialize_non_transferable;
mod initialize_permanent_delegate;
mod initialize_transfer_fee_config;
mod initialize_transfer_hook;

pub use initialize_close_authority::*;
pub use initialize_group_member_pointer::*;
pub use initialize_group_pointer::*;
pub use initialize_immutable_owner::*;
pub use initialize_metadata_pointer::*;
pub use initialize_non_transferable::*;
pub use initialize_permanent_delegate::*;
pub use initialize_transfer_fee_config::*;
pub use initialize_transfer_hook::*;

use core::mem::size_of;

use crate::pubkey::Pubkey;

const NONE_PUBKEY: Pubkey = Pubkey::new_from_array([0; 32]);

fn unwrap_or_default_pubkey(key: Option<&Pubkey>) -> &Pubkey {
    key.unwrap_or(&NONE_PUBKEY)
}

const IX_INITIALIZE_POINTER_DATA_LEN: usize = size_of::<u8>() // token instruction selector
    + size_of::<u8>() // pointer instruction selector
    + size_of::<Pubkey>() // authority
    + size_of::<Pubkey>(); // pointer

fn serialize_initialize_pointer_instruction_data(
    selector: u8,
    authority: &Pubkey,
    pointer: &Pubkey,
) -> [u8; 66] {
    let mut instruction_data = [0; IX_INITIALIZE_POINTER_DATA_LEN];

    instruction_data[0] = selector;

    // Initialize extension pointer selector == 0, so no need to set it.
    instruction_data[2..34].copy_from_slice(&authority.to_bytes());
    instruction_data[34..66].copy_from_slice(&pointer.to_bytes());

    instruction_data
}
