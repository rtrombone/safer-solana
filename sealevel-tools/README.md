# Sealevel Tools

This crate is not an attempt to create a new framework for writing Solana programs. Instead, it is a
set of tools that should help a developer do whatever he wants to do in a Solana program without
prescribing any specific way of doing so. By using these tools, a developer can write a lightweight
program with functionality found in other frameworks.

Currently pinning Solana version to 1.18. Looking to support Solana 2.0 in the future.

## Examples

Here are some ways of using these tools to write your first program.

### Instruction Selectors

Frameworks like `anchor-lang` and `spl-discriminator` prescribe that the first 8 bytes of a Sha256
hash representing the name of a given instruction should be used to determine how instruction data
should be processed in your program.

For example, `anchor-lang` typically uses the input "global:your_instruction_name" to generate the
Sha256 hash. This can be achieved using the [Discriminator](src/discriminator/mod.rs) enum:

```rs
const YOUR_INSTRUCTION_SELECTOR: [u8; 8] =
    Discriminator::Sha2(b"global:your_instruction_name").to_bytes()
```

Maybe you believe these selectors do not have to be so large as the collision among your
instructions is nearly zero. You can make a 4-byte selector similarly:

```rs
const YOUR_INSTRUCTION_SELECTOR: [u8; 4] =
    Discriminator::Sha2(b"ix::your_instruction_name").to_bytes()
```

Or use a different hashing computation incorporating the arguments for your instruction (like how
Solidity works).

```rs
const YOUR_INSTRUCTION_SELECTOR: [u8; 4] =
    Discriminator::Keccak(b"your_instruction_name(u64,Pubkey)").to_bytes()
```

Usually it is nice to store your instructions in an enum:

```rs
#[derive(Debug)]
pub enum ProgramInstruction {
    DoSomething(u64),
    AddThing(ThingArgs),
    RemoveThing,
    DoSomethingElse { a: u32, b: [u8; 12] }
}
```

Implementing the constant selectors is a nice way to build these into your program binary as consts.
Then your processor can take the deserialized arguments of each instruction.

```rs
impl ProgramInstruction {
    pub const DO_SOMETHING: [u8; 4] = Discriminator::Sha2(b"do_something").to_bytes();
    pub const ADD_THING: [u8; 4] = Discriminator::Sha2(b"add_thing").to_bytes();
    pub const REMOVE_THING: [u8; 4] = Discriminator::Sha2(b"remove_thing").to_bytes();
    pub const DO_SOMETHING_ELSE: [u8; 4] = Discriminator::Sha2(b"do_something_else").to_bytes();

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        if program_id != &ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        match BorshDeserialize::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?
        {
            Self::DoSomething(data) => {
                msg!("DoSomething: {}", data);
            }
            Self::AddThing(_) => {
                msg!("AddThing");
            }
            Self::RemoveThing => {
                msg!("RemoveThing");
            }
            Self::DoSomethingElse { a, b } => {
                msg!("DoSomethingElse: a={}, b={:?}", a, b);
            }
        }

        Ok(())
    }
}

impl BorshDeserialize for ProgramInstruction {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        match BorshDeserialize::deserialize_reader(reader)? {
            Self::DO_SOMETHING => Ok(Self::DoSomething(BorshDeserialize::deserialize_reader(
                reader,
            )?)),
            Self::ADD_THING => Ok(Self::AddThing(BorshDeserialize::deserialize_reader(
                reader,
            )?)),
            Self::REMOVE_THING => Ok(Self::RemoveThing),
            Self::DO_SOMETHING_ELSE => Ok(Self::DoSomethingElse {
                a: BorshDeserialize::deserialize_reader(reader)?,
                b: BorshDeserialize::deserialize_reader(reader)?,
            }),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid discriminator",
            )),
        }
    }
}

impl BorshSerialize for ProgramInstruction {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Self::DoSomething(data) => {
                Self::DO_SOMETHING.serialize(writer)?;
                data.serialize(writer)
            }
            Self::AddThing(args) => {
                Self::ADD_THING.serialize(writer)?;
                args.serialize(writer)
            }
            Self::RemoveThing => Self::REMOVE_THING.serialize(writer),
            Self::DoSomethingElse { a, b } => {
                Self::DO_SOMETHING_ELSE.serialize(writer)?;
                a.serialize(writer)?;
                b.serialize(writer)
            }
        }
    }
}
```

Instead of just logging using `msg!`, you would use a processor method relevant for each instruction
(e.g. matching `DoSomething` would call an internal method resembling
`fn process_do_something(accounts: &[AccountInfo], data: u64)`).

## Accounts

Without using a framework, the `AccountInfo` slice's iterator is used in conjunction with
`next_account_info` to take the next `AccountInfo` from this slice.

With a framework like `anchor-lang`, these accounts are defined upfront in a struct, which derives
the `Accounts` trait:

```rs
#[derive(Accounts)]
pub struct AddThing<'a> {
    #[account(mut)]
    payer: Signer<'a>,

    #[account(
        init,
        payer = payer,
        space = 16,
        seeds = [b"thing"],
        bump,
    )]
    new_thing: Account<'a, Thing>,

    system_program: Program<'a, System>,
}
```

And `Thing` account schema is defined as:

```rs
#[account]
#[derive(Debug, PartialEq, Eq)]
pub struct Thing {
    pub data: u64,
}
```

Using these tools, accounts can be plucked off in the processor method or accounts can be contained
in a struct similar to how Solana program frameworks organize them.

Without a struct, you may iterate like so:

```rs
    let mut accounts_iter = accounts.iter().enumerate();

    // First account will be paying the rent.
    let from_pubkey =
        try_next_enumerated_account_as::<Signer<true>>(&mut accounts_iter, Default::default())
            .map(|(_, signer)| signer.key)?;

    let (new_thing_addr, new_thing_bump) = Pubkey::find_program_address(&[b"thing"], program_id);

    // Second account is the new Thing.
    let (_, new_thing_account) = try_next_enumerated_account_as::<DataAccount<true>>(
        &mut accounts_iter,
        NextEnumeratedAccountOptions {
            key: Some(&new_thing_addr),
            ..Default::default()
        },
    )?;
```

`try_next_enumerated_account_as<T>` takes an enumerated iterator and returns tools-defined types,
which are simple wrappers around `AccountInfo` (e.g. `Signer<const WRITE: bool>` where `WRITE`
defines whether the account is writable). `NextEnumeratedAccountOptions` provide some optional
constraints when plucking off the next account (e.g. verifying that the pubkey equals what you
expect). In the above example, we are asserting that the new `Thing` account is a
`DataAccount<true>`, whose const bool value says that it is a writable account.

If you desire more structure in your life, encapsulate the account plucking logic in a struct:

```rs
struct AddThingAccounts<'a, 'b> {
    payer: (usize, Signer<'a, 'b, true>),
    new_thing: (
        usize,
        DataAccount<'a, 'b, true>,
        u8, // bump
    ),
}

impl<'a, 'b> AddThingAccounts<'a, 'b> {
    fn try_new(
        accounts: &'b [AccountInfo<'a>],
        program_id: &'b Pubkey,
    ) -> Result<Self, ProgramError> {
        let mut accounts_iter = accounts.iter().enumerate();

        let payer = try_next_enumerated_account_as(&mut accounts_iter, Default::default())?;

        let (new_thing_addr, new_thing_bump) =
            Pubkey::find_program_address(&[b"thing"], program_id);

        let (new_thing_index, new_thing_account) = try_next_enumerated_account_as(
            &mut accounts_iter,
            NextEnumeratedAccountOptions {
                key: Some(&new_thing_addr),
                ..Default::default()
            },
        )?;

        Ok(Self {
            payer,
            new_thing: (new_thing_index, new_thing_account, new_thing_bump),
        })
    }
}
```

Account indices are helpful when a particular account has an error (where you can revert with a
colorful error message indicating which account is the culprit). Solana program frameworks just give
a pubkey or name of the account that failed, which are helpful relative to the IDL these SDKs
leverage. But when writing a program with these tools, the next best option is giving the index of
the accounts array you passed into your transaction. `try_next_enumerated_account_as` has error
handling that gives the user information about which account index failed any checks using the
`NextEnumeratedAccountOptions`.

Also notice that we do not check that the System program is provided. You can add an explicit check
for it (like how `anchor-lang` requires it). Or it can be assumed that it is one of the remaining
accounts in the `AccountInfo` slice since the `Thing` being created would fail without it (since the
CPI call to the System program requires it).

To wrap up this example, because `Thing` is a new account, you can create it like so:

```rs
    try_create_borsh_data_account(
        CreateAccount {
            from_pubkey,
            to: new_thing_account.deref().into(),
            space: 16,
            program_id,
            account_infos: accounts,
            from_signer_seeds: None,
            to_signer_seeds: Some(&[b"thing", &[new_thing_bump]]),
        },
        &Thing { data: 69 },
        Some(&Thing::DISCRIMINATOR),
    )?;
```

And `Thing` account schema in the lightweight example is defined as:

```rs
#[derive(Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub struct Thing {
    pub data: u64,
}

impl Thing {
    pub const DISCRIMINATOR: [u8; 8] = Discriminator::Sha2(b"account:Thing").to_bytes();
}
```

The account discriminator does not have to be 8 bytes like how `anchor-lang` and `spl-discriminator`
enforce it to be. To save on a bit of rent, 4 bytes should be sufficient to avoid collision among
all of your program's data accounts (where the cost savings is 4 * 6,960 lamports).

There are more lines of code required to perform the same functionality that Solana program
framework may remove from your life. For example, `anchor-lang` would only require this to
instantiate your `Thing`:
```rs
    pub fn add_thing(ctx: Context<AddThing>) -> Result<()> {
        ctx.accounts.new_thing.set_inner(Thing { data: 69 });

        Ok(())
    }
```

But in an attempt to keeping things simple and lightweight, the cost is a huge increase in program
binary size and requiring more compute units than necessary to perform the same task. Pick your
poison. But larger binary size translates to a higher deployment cost and higher compute units can
affect your end users.

## Philosophy

The tools found in this crate are meant to allow a developer to keep things as simple as possible
while providing some guardrails. These guardrails are not meant to enforce any specific way of
iterating through entrypoint accounts, account serialization/deserialization (serde), how
instruction discriminators should be set, etc.

The developer should write his program however he wants. For example, a developer may want his
instruction selectors to be the first 4-bytes of a Keccak256 hash similar to how Solidity for EVM
works. Or to be consistent with how `anchor-lang` and `spl-discriminator` define discriminators as
the first 8-bytes of Sha256 (sha2).

What this crate does not attempt to do is generate an IDL. While convenient when using frameworks
like `anchor-lang` and `shank` (where a front-end language-agnostic developer can take an IDL and
build an SDK using it), these tools are meant to focus on safer program development. Writing
instruction builders is trivial, and any time spent trying to resolve headaches having an IDL build
using any of these frameworks can be saved by writing custom instruction builders in whichever
language you want to support.

Solana program frameworks attempt to remove boilerplate from writing instruction processors. But
with that comes the price of having to learn how these specific macros work. And these macros can
add a lot of bloat to your program, where your program size can easily be double the size of a
program with the same logic but without a specific framework.