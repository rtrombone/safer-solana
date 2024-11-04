//! CPI for System program.
//!
//! ### Notes
//!
//! This module does not have optimized CPI calls for every instruction. For any instruction you
//! need for your program that does not exist here, please use [invoke_signed] with instruction
//! builders found in the [solana_program::system_instruction] module.
//!
//! See detailed examples of how to perform CPI with [CreateAccount].
//!
//! [invoke_signed]: crate::cpi::invoke_signed

mod allocate;
mod assign;
mod create_account;
mod transfer;

pub use allocate::*;
pub use assign::*;
pub use create_account::*;
pub use transfer::*;

pub use solana_program::system_program::ID;
