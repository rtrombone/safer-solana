//! This crate is not an attempt to create a new framework for writing Solana programs. Instead, it
//! is a set of tools that should help a developer write a Solana program without prescribing any
//! specific way of doing so. By using these tools, a developer can write a lightweight program with
//! functionality found in other frameworks.
//!
//! Currently, this package leverages [sealevel_nostd_entrypoint], which is a fork of an optimized
//! no-std program entrypoint library. Its contents are re-exported for convenience.
//! ```
//! use sealevel_tools::{
//!     entrypoint::{NoStdAccountInfo, ProgramResult, entrypoint_nostd},
//!     log::sol_log,
//!     pubkey::Pubkey,
//! };
//!
//! pub fn process_instruction(
//!     program_id: &Pubkey,
//!     accounts: &[NoStdAccountInfo],
//!     instruction_data: &[u8],
//! ) -> ProgramResult {
//!     // TODO: Check your program ID.
//!     let _ = program_id;
//!
//!     // TODO: Check and use your accounts.
//!     let _ = accounts;
//!
//!     // TODO: Check and use your data.
//!     let _ = instruction_data;
//!
//!     sol_log("Hello, world!");
//!
//!     Ok(())
//! }
//!
//! entrypoint_nostd!(process_instruction, 8);
//! ```
//!
//! See this crate's [README] for more information about MSRV and feature flags.
//!
//! # Examples
//!
//! Check out the [safer-solana] repository for [working examples] of using this
//! package. Below are some rudimentary examples of how to use some of these
//! tools.
//!
//! # Details
//!
//! Here are some ways of using these tools to write your first program.
//!
//! ## Instruction Selectors
//!
//! Frameworks like [anchor-lang] and [spl-discriminator] prescribe that the first 8 bytes of a
//! Sha256 hash representing the name of a given instruction should be used to determine how
//! instruction data should be processed in your program.
//!
//! For example, [anchor-lang] typically uses the input "global:your_instruction_name" to generate
//! the Sha256 hash. This can be achieved using [Discriminator]:
//! ```
//! # use sealevel_tools::discriminator::Discriminator;
//! #
//! const YOUR_INSTRUCTION_SELECTOR: [u8; 8] =
//!     Discriminator::Sha2(b"global:your_instruction_name").to_bytes();
//! ```
//!
//! Maybe you believe these selectors do not have to be so large as the collision among your
//! instructions is nearly zero. You can make a 4-byte selector similarly:
//! ```
//! # use sealevel_tools::discriminator::Discriminator;
//! #
//! const YOUR_INSTRUCTION_SELECTOR: [u8; 4] =
//!     Discriminator::Sha2(b"ix::your_instruction_name").to_bytes();
//! ```
//!
//! Or use a different hashing computation incorporating the arguments for your instruction (like
//! how Solidity works).
//! ```
//! # use sealevel_tools::discriminator::Discriminator;
//! #
//! const YOUR_INSTRUCTION_SELECTOR: [u8; 4] =
//!     Discriminator::Keccak(b"your_instruction_name(u64,Pubkey)").to_bytes();
//! ```
//!
//! Usually it is nice to store your instructions in an enum. Implementing the constant selectors is
//! a nice way to build these into your program binary as consts. Then your processor can take the
//! deserialized arguments of each instruction. NOTE: This example uses [borsh] for serde, but your
//! program is not required to use it to decode instruction data.
//! ```
//! use sealevel_tools::{
//!     borsh::{io, BorshDeserialize, BorshSerialize},
//!     discriminator::Discriminator,
//!     entrypoint::{entrypoint_nostd, NoStdAccountInfo, ProgramResult},
//!     msg,
//!     program_error::ProgramError,
//!     pubkey::Pubkey,
//! };
//!
//! sealevel_tools::declare_id!("Examp1eThing1111111111111111111111111111111");
//!
//! #[derive(Debug, BorshDeserialize, BorshSerialize)]
//! # pub struct ThingArgs(u32);
//!
//! #[derive(Debug)]
//! pub enum ProgramInstruction {
//!     DoSomething(u64),
//!     AddThing(ThingArgs),
//!     RemoveThing,
//!     DoSomethingElse { a: u32, b: [u8; 12] }
//! }
//!
//! pub type Selector = [u8; 4];
//!
//! impl ProgramInstruction {
//!     pub const DO_SOMETHING: Selector = Discriminator::Sha2(b"do_something").to_bytes();
//!     pub const ADD_THING: Selector = Discriminator::Sha2(b"add_thing").to_bytes();
//!     pub const REMOVE_THING: Selector = Discriminator::Sha2(b"remove_thing").to_bytes();
//!     pub const DO_SOMETHING_ELSE: Selector =
//!         Discriminator::Sha2(b"do_something_else").to_bytes();
//! }
//!
//! impl BorshDeserialize for ProgramInstruction {
//!     fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
//!         match BorshDeserialize::deserialize_reader(reader)? {
//!             Self::DO_SOMETHING => Ok(Self::DoSomething(BorshDeserialize::deserialize_reader(
//!                 reader,
//!             )?)),
//!             Self::ADD_THING => Ok(Self::AddThing(BorshDeserialize::deserialize_reader(
//!                 reader,
//!             )?)),
//!             Self::REMOVE_THING => Ok(Self::RemoveThing),
//!             Self::DO_SOMETHING_ELSE => Ok(Self::DoSomethingElse {
//!                 a: BorshDeserialize::deserialize_reader(reader)?,
//!                 b: BorshDeserialize::deserialize_reader(reader)?,
//!             }),
//!             _ => Err(io::Error::new(
//!                 io::ErrorKind::InvalidData,
//!                 "Invalid discriminator",
//!             )),
//!         }
//!     }
//! }
//!
//! impl BorshSerialize for ProgramInstruction {
//!     fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
//!         match self {
//!             Self::DoSomething(data) => {
//!                 Self::DO_SOMETHING.serialize(writer)?;
//!                 data.serialize(writer)
//!             }
//!             Self::AddThing(args) => {
//!                 Self::ADD_THING.serialize(writer)?;
//!                 args.serialize(writer)
//!             }
//!             Self::RemoveThing => Self::REMOVE_THING.serialize(writer),
//!             Self::DoSomethingElse { a, b } => {
//!                 Self::DO_SOMETHING_ELSE.serialize(writer)?;
//!                 a.serialize(writer)?;
//!                 b.serialize(writer)
//!             }
//!         }
//!     }
//! }
//!
//! pub fn process_instruction(
//!     program_id: &Pubkey,
//!     accounts: &[NoStdAccountInfo],
//!     instruction_data: &[u8],
//! ) -> ProgramResult {
//!     if program_id != &ID {
//!         return Err(ProgramError::IncorrectProgramId);
//!     }
//!
//!     match BorshDeserialize::try_from_slice(instruction_data)
//!         .map_err(|_| ProgramError::InvalidInstructionData)?
//!     {
//!         ProgramInstruction::DoSomething(data) => {
//!             msg!("DoSomething: {}", data);
//!         }
//!         ProgramInstruction::AddThing(_) => {
//!             msg!("AddThing");
//!         }
//!         ProgramInstruction::RemoveThing => {
//!             msg!("RemoveThing");
//!         }
//!         ProgramInstruction::DoSomethingElse { a, b } => {
//!             msg!("DoSomethingElse: a={}, b={:?}", a, b);
//!         }
//!     }
//!
//!     Ok(())
//! }
//!
//! entrypoint_nostd!(process_instruction, 8);
//! ```
//!
//! Instead of just logging using [msg!], you would use a processor method relevant for each
//! instruction. For example, matching `DoSomething` would call an internal method resembling:
//! ```
//! # use sealevel_tools::entrypoint::{NoStdAccountInfo, ProgramResult};
//! #
//! fn process_do_something(accounts: &[NoStdAccountInfo], data: u64) -> ProgramResult {
//!     // Do something useful here.
//!     Ok(())
//! }
//! ```
//!
//! ## Accounts
//!
//! Without using a framework, the account slice's iterator is used in conjunction with
//! [next_account_info] to take the next account from this slice.
//!
//! With a framework like [anchor-lang], these accounts are defined upfront in a struct, which
//! derives the [Accounts] trait:
//! ```ignore
//! #[derive(Accounts)]
//! pub struct AddThing<'a> {
//!     #[account(mut)]
//!     payer: Signer<'a>,
//!
//!     #[account(
//!         init,
//!         payer = payer,
//!         space = 16,
//!         seeds = [b"thing"],
//!         bump,
//!     )]
//!     new_thing: Account<'a, Thing>,
//!
//!     system_program: Program<'a, System>,
//! }
//! ```
//!
//! And `Thing` account schema is defined as:
//! ```ignore
//! #[account]
//! #[derive(Debug, PartialEq, Eq)]
//! pub struct Thing {
//!     pub data: u64,
//! }
//! ```
//!
//! Using these tools, accounts can be plucked off in the processor method or accounts can be contained
//! in a struct similar to how Solana program frameworks organize them.
//!
//! Without a struct, you may iterate like so:
//! ```
//! # use sealevel_tools::{
//! #   account_info::{
//! #       try_next_enumerated_account, AccountInfoConstraints, Payer, WritableAccount
//! #   },
//! #   entrypoint::{NoStdAccountInfo, ProgramResult},
//! #   pubkey::Pubkey,
//! # };
//! #
//! # sealevel_tools::declare_id!("Examp1eThing1111111111111111111111111111111");
//! #
//! fn process(accounts: &[NoStdAccountInfo]) -> ProgramResult {
//!     let mut accounts_iter = accounts.iter().enumerate();
//!
//!     // First account will be paying the rent.
//!     let (_, payer) =
//!         try_next_enumerated_account::<Payer>(&mut accounts_iter, Default::default())?;
//!
//!     let (new_thing_addr, new_thing_bump) =
//!         Pubkey::find_program_address(&[b"thing"], &ID);
//!
//!     // Second account is the new Thing.
//!     let (_, new_thing_account) = try_next_enumerated_account::<WritableAccount>(
//!         &mut accounts_iter,
//!         AccountInfoConstraints {
//!             key: Some(&new_thing_addr),
//!             ..Default::default()
//!         },
//!     )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! [try_next_enumerated_account] takes an enumerated iterator and
//! returns tools-defined types, which are simple wrappers around [NoStdAccountInfo] (e.g.
//! [Payer], which is a writable [Signer]. [AccountInfoConstraints] provide some optional
//! constraints when plucking off the next account (e.g. verifying that the pubkey equals what you
//! expect). In the above example, we are asserting that the new `Thing` account is a
//! [WritableAccount], whose const bool value says that it is a writable account.
//!
//! If you desire more structure in your life, encapsulate the account plucking logic in a struct
//! via the [TakeAccounts] trait:
//!
//! ```
//! # use sealevel_tools::{
//! #   account_info::{
//! #       try_next_enumerated_account, AccountInfoConstraints, Payer, TakeAccounts,
//! #       WritableAccount
//! #   },
//! #   entrypoint::NoStdAccountInfo,
//! #   program_error::ProgramError,
//! #   pubkey::Pubkey,
//! # };
//! #
//! # sealevel_tools::declare_id!("Examp1eThing1111111111111111111111111111111");
//! #
//! struct AddThingAccounts<'a> {
//!     payer: (usize, Payer<'a>),
//!     new_thing: (
//!         usize,
//!         WritableAccount<'a>,
//!         u8, // bump
//!     ),
//! }
//!
//! impl<'a> TakeAccounts<'a> for AddThingAccounts<'a> {
//!     fn take_accounts(
//!         iter: &mut impl Iterator<Item = (usize, &'a NoStdAccountInfo)>,
//!     ) -> Result<Self, ProgramError> {
//!         let payer = try_next_enumerated_account(iter, Default::default())?;
//!
//!         let (new_thing_addr, new_thing_bump) =
//!             Pubkey::find_program_address(&[b"thing"], &ID);
//!
//!         let (new_thing_index, new_thing_account) = try_next_enumerated_account(
//!             iter,
//!             AccountInfoConstraints {
//!                 key: Some(&new_thing_addr),
//!                 ..Default::default()
//!             },
//!         )?;
//!
//!         Ok(Self {
//!             payer,
//!             new_thing: (new_thing_index, new_thing_account, new_thing_bump),
//!         })
//!     }
//! }
//! ```
//!
//! Account indices are helpful when a particular account has an error (where you can revert with a
//! colorful error message indicating which account is the culprit). Solana program frameworks just
//! give a pubkey or name of the account that failed, which are helpful relative to the IDL these
//! SDKs leverage. But when writing a program with these tools, the next best option is giving the
//! index of the accounts array you passed into your transaction. [try_next_enumerated_account] has
//! error handling that gives the user information about which account index failed any checks using
//! the [AccountInfoConstraints].
//!
//! Also notice that we do not check that the System program is provided. You can add an explicit
//! check for it (like how [anchor-lang] requires it). Or it can be assumed that it is one of the
//! remaining accounts in the [NoStdAccountInfo] slice since the `Thing` being created would fail
//! without it (since the CPI call to the System program requires it).
//!
//! To wrap up this example, because `Thing` is a new account, you can create it like so:
//! ```
//! # use borsh::{BorshDeserialize, BorshSerialize};
//! # use sealevel_tools::{
//! #    account::{AccountSerde, BorshAccountSchema},
//! #    account_info::{
//! #       try_next_enumerated_account, AccountInfoConstraints, Payer, WritableAccount
//! #    },
//! #    cpi::system_program::CreateAccount,
//! #    discriminator::{Discriminate, Discriminator},
//! #    entrypoint::{NoStdAccountInfo, ProgramResult},
//! #    pubkey::Pubkey,
//! # };
//! #
//! # sealevel_tools::declare_id!("Examp1eThing1111111111111111111111111111111");
//! #
//! #[derive(Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
//! pub struct Thing {
//!     pub data: u64,
//! }
//!
//! impl Discriminate<8> for Thing {
//!     const DISCRIMINATOR: [u8; 8] = Discriminator::Sha2(b"account:Thing").to_bytes();
//! }
//!
//! fn process(accounts: &[NoStdAccountInfo]) -> ProgramResult {
//! #     let mut accounts_iter = accounts.iter().enumerate();
//! #
//! #     let (_, payer) =
//! #         try_next_enumerated_account::<Payer>(&mut accounts_iter, Default::default())?;
//! #
//! #     let (new_thing_addr, new_thing_bump) =
//! #         Pubkey::find_program_address(&[b"thing"], &ID);
//! #
//! #     let (_, new_thing_account) = try_next_enumerated_account::<WritableAccount>(
//! #         &mut accounts_iter,
//! #         AccountInfoConstraints {
//! #             key: Some(&new_thing_addr),
//! #             ..Default::default()
//! #         },
//! #     )?;
//! #
//!     CreateAccount {
//!         payer: payer.as_cpi_authority(),
//!         to: new_thing_account.as_cpi_authority(Some(&[b"thing", &[new_thing_bump]])),
//!         program_id: &ID,
//!         space: None,
//!         lamports: None,
//!     }
//!     .try_invoke_and_serialize(&BorshAccountSchema(Thing { data: 69 }))?;
//! #
//! #   Ok(())
//! }
//! ```
//!
//! The account discriminator does not have to be 8 bytes like how [anchor-lang] and
//! [spl-discriminator] enforce it to be. To save on a bit of rent, 4 bytes should be sufficient to
//! avoid collision among all of your program's data accounts (where the cost savings is 4 * 6,960
//! lamports).
//!
//! There are more lines of code required to perform the same functionality that Solana program
//! framework may remove from your life. For example, [anchor-lang] would only require this to
//! instantiate your `Thing`:
//! ```ignore
//! pub fn add_thing(ctx: Context<AddThing>) -> Result<()> {
//!     ctx.accounts.new_thing.set_inner(Thing { data: 69 });
//!     Ok(())
//! }
//! ```
//!
//! But in an attempt to keeping things simple and lightweight, the cost is a huge increase in program
//! binary size and requiring more compute units than necessary to perform the same task. Pick your
//! poison. But larger binary size translates to a higher deployment cost and higher compute units can
//! affect your end users.
//!
//! [AccountInfo]: https://docs.rs/solana-account-info/latest/solana_account_info/struct.AccountInfo.html
//! [Accounts]: https://docs.rs/anchor-lang/latest/anchor_lang/trait.Accounts.html
//! [Discriminator]: crate::discriminator::Discriminator
//! [AccountInfoConstraints]: crate::account_info::AccountInfoConstraints
//! [NoStdAccountInfo]: crate::entrypoint::NoStdAccountInfo
//! [Payer]: crate::account_info::Payer
//! [README]: https://crates.io/crates/sealevel-tools
//! [Signer]: crate::account_info::Signer
//! [TakeAccounts]: crate::account_info::TakeAccounts
//! [WritableAccount]: crate::account_info::WritableAccount
//! [anchor-lang]: https://docs.rs/anchor-lang/latest/anchor_lang/
//! [msg!]: https://docs.rs/solana-msg/latest/solana_msg/macro.msg.html
//! [next_account_info]: https://docs.rs/solana-account-info/latest/solana_account_info/fn.next_account_info.html
//! [safer-solana]: https://github.com/rtrombone/safer-solana
//! [spl-discriminator]: https://docs.rs/spl-discriminator/latest/spl_discriminator/
//! [shank]: https://docs.rs/shank/latest/shank/
//! [try_next_enumerated_account]: crate::account_info::try_next_enumerated_account
//! [working examples]: https://github.com/rtrombone/safer-solana/tree/main/examples/

#![deny(dead_code, unused_imports, unused_mut, unused_variables)]
#![no_std]

pub mod account;
pub mod account_info;
pub mod cpi;
pub mod discriminator;
mod error;
pub mod log;
pub mod pda;
pub mod sysvar;

pub use error::SealevelToolsError;

/// Re-export of [sealevel_nostd_entrypoint] items.
///
/// ### Notes
///
/// Because our package leverages this optimized no-std Solana entrypoint crate for account and CPI
/// handling, its contents are re-exported here for convenience (so there is no need to add
/// [sealevel_nostd_entrypoint] as a dependency in your program).
pub mod entrypoint {
    pub use sealevel_nostd_entrypoint::{
        basic_panic_impl, deserialize_nostd, deserialize_nostd_no_dup,
        deserialize_nostd_no_dup_no_program, deserialize_nostd_no_program, entrypoint_nostd,
        entrypoint_nostd_no_duplicates, entrypoint_nostd_no_duplicates_no_program,
        entrypoint_nostd_no_program, noalloc_allocator, AccountInfoC, AccountMetaC, InstructionC,
        NoStdAccountInfo, NoStdAccountInfoInner, RcRefCellInner, Ref, RefMut,
    };

    pub use crate::program_error::ProgramResult;
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "borsh")]
pub use borsh;
pub use solana_msg::msg;
pub use solana_program_error as program_error;
pub use solana_pubkey::{self as pubkey, declare_id, pubkey};
#[cfg(feature = "token")]
pub use spl_token_2022;
