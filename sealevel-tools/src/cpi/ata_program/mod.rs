//! CPI for an Associated Token Account program (including the canonical ATA program).
//!
//! ### Notes
//!
//! If there is another program that uses the same ATA program interface, you can use the same CPI
//! calls by specifying that program ID.
//!
//! This module does not have optimized CPI calls for every instruction. For any instruction you
//! need for your program that does not exist here, please use [invoke_signed] with instruction
//! builders found in the [spl_associated_token_account] crate.
//!
//! [invoke_signed]: crate::cpi::invoke_signed
//! [spl_associated_token_account]: <https://docs.rs/spl-associated-token-account/latest/spl_associated_token_account/instruction/index.html>

mod create;

pub use create::*;

crate::declare_id!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
